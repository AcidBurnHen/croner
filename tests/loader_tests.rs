use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use croner::{
    loader::{load_config, ConfigCache},
    models::Fanout,
};

fn temp_path(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    p.push(format!("{}_{}", name, nanos));
    p
}

fn write(path: &Path, s: &str) {
    let mut f = File::create(path).expect("create failed");
    f.write_all(s.as_bytes()).expect("write failed");
    // ensure mtime changes on most filesystems for reload tests
    sleep(Duration::from_millis(5));
}

#[test]
fn loads_basic_jobs_with_int_and_list_fanout() {
    let p = temp_path("config.croner");
    write(
        &p,
        r#"
[job:index_articles]
schedule = */15 * * * *
command = python index.py
fanout = 4

[job:daily_etl]
schedule = 0 2 * * *
command = python etl.py
fanout = ["--source=internal --mode=full", "--source=external --mode=delta"]

[job:ping]
schedule = * * * * *
command = python ping.py
"#,
    );

    let jobs = load_config(&p).expect("should parse");
    assert_eq!(jobs.len(), 3);

    // Job 0: int fanout
    assert_eq!(jobs[0].id, "index_articles");
    match jobs[0].fanout {
        Fanout::Int(4) => {}
        _ => panic!("expected Fanout::Int(4)"),
    }

    // Job 1: list fanout
    match jobs[1].fanout {
        Fanout::List(ref v) => {
            assert_eq!(v.len(), 2, "expected two fanout entries");
            assert!(v.iter().any(|args| {
                args.iter()
                    .any(|a| a.to_string_lossy().contains("--source=internal"))
            }));
            assert!(v.iter().any(|args| {
                args.iter()
                    .any(|a| a.to_string_lossy().contains("--source=external"))
            }));
        }
        _ => panic!("expected Fanout::List"),
    }

    // Job 2: no fanout
    match jobs[2].fanout {
        Fanout::None => {}
        _ => panic!("expected Fanout::None"),
    }
}

#[test]
fn ignores_comments_blank_lines_and_handles_crlf() {
    let p = temp_path("conf.croner");
    let content = "[job:a]\r\n \
                   # comment\r\n\
                   schedule = * * * * *\r\n\
                   command = echo \"hi\"\r\n";
    write(&p, content);

    let jobs = load_config(&p).expect("parse");
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, "a");
    assert!(matches!(jobs[0].fanout, Fanout::None));
}

#[test]
fn handles_utf8_bom() {
    let p = temp_path("bom.croner");
    let bom = [0xEFu8, 0xBB, 0xBF];
    let body = br#"[job:x]
schedule = * * * * *
command = echo hi
"#;
    let mut f = File::create(&p).unwrap();
    f.write_all(&bom).unwrap();
    f.write_all(body).unwrap();
    drop(f);
    sleep(Duration::from_millis(5));

    let jobs = load_config(&p).expect("parse with BOM");
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, "x");
}

#[test]
fn error_on_unknown_key() {
    let p = temp_path("bad.croner");
    write(
        &p,
        r#"
[job:a]
schedule = * * * * *
command = echo hi
wat = huh
"#,
    );

    let err = load_config(&p).unwrap_err();
    assert!(err.to_lowercase().contains("unknown key"));
}

#[test]
fn error_on_duplicate_key() {
    let p = temp_path("dupkey.croner");
    write(
        &p,
        r#"
[job:a]
schedule = * * * * *
schedule = * * * * *
command = echo hi
"#,
    );

    let err = load_config(&p).unwrap_err();
    assert!(err.to_lowercase().contains("duplicate `schedule`"));
}

#[test]
fn error_on_missing_required_fields() {
    let p = temp_path("missing.croner");
    write(
        &p,
        r#"
[job:a]
command = echo hi
"#,
    );

    let err = load_config(&p).unwrap_err();
    assert!(err.to_lowercase().contains("missing schedule"));

    write(
        &p,
        r#"
[job:a]
schedule = * * * * *
"#,
    );
    let err = load_config(&p).unwrap_err();
    assert!(err.to_lowercase().contains("missing command"));
}

#[test]
fn error_on_duplicate_fanout_key() {
    let p = temp_path("fanout_conflict.croner");
    write(
        &p,
        r#"
[job:a]
schedule = * * * * *
command = echo hi
fanout = 2
fanout = ["--x"]
"#,
    );
    let err = load_config(&p).unwrap_err();
    assert!(err.to_lowercase().contains("duplicate `fanout`"));
}

#[test]
fn error_on_fanout_non_int() {
    let p = temp_path("fanout_nonint.croner");
    write(
        &p,
        r#"
[job:a]
schedule = * * * * *
command = echo hi
fanout = nope
"#,
    );
    let err = load_config(&p).unwrap_err();
    assert!(err
        .to_lowercase()
        .contains("fanout must be an integer or list"));
}

#[test]
fn error_on_duplicate_ids() {
    let p = temp_path("dupid.croner");
    write(
        &p,
        r#"
[job:a]
schedule = * * * * *
command = echo hi

[job:a]
schedule = * * * * *
command = echo hi
"#,
    );

    let err = load_config(&p).unwrap_err();
    assert!(err.to_lowercase().contains("duplicate job id"));
}

#[test]
fn cache_reload_if_changed_behaves_atomically() {
    let p = temp_path("live.croner");
    write(
        &p,
        r#"
[job:a]
schedule = * * * * *
command = echo hi
"#,
    );

    let mut cache = ConfigCache::new();

    // first load → true, jobs populated
    let changed = cache.reload_if_changed(&p).expect("reload");
    assert!(changed);
    assert_eq!(cache.jobs.len(), 1);
    assert_eq!(cache.jobs[0].id, "a");

    // unchanged → false
    let changed = cache.reload_if_changed(&p).expect("reload unchanged");
    assert!(!changed);
    assert_eq!(cache.jobs.len(), 1);

    // corrupt file → reload returns Err, cache.jobs unchanged
    write(
        &p,
        r#"
[job:a]
schedule = * * * * *
# missing command
"#,
    );
    let res = cache.reload_if_changed(&p);
    assert!(res.is_err(), "should error on invalid config");
    assert_eq!(cache.jobs.len(), 1);
    assert_eq!(cache.jobs[0].id, "a");

    // fix file → reload true, jobs updated
    write(
        &p,
        r#"
[job:b]
schedule = * * * * *
command = echo bye
"#,
    );
    let changed = cache.reload_if_changed(&p).expect("reload fixed");
    assert!(changed);
    assert_eq!(cache.jobs.len(), 1);
    assert_eq!(cache.jobs[0].id, "b");
}
