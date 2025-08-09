# Croner 

- Work in progress 
- Docs will be updated when the lib is finished 
- A rewrite of pycroner in Rust 

- Note: This started of as a complete rewrite of pycroner, but as I am reworking this library in Rust and thinking about performance benefits, there will be some big changes to how the logic works going forward. And I plan to bring all the impactful changes into pycroner as well. 

The first change is that we will be ditching the polling runner logic, and going for a scheduler based approach. 