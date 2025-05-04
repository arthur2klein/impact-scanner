# Impact-Scanner

## Description

This project's final goal are:
- [x] identifying symbols from staged changes in a git repository,
- [ ] deducing symbols affected throughout the project,
- [ ] selecting test functions from these symbols,
- [ ] run these tests.

Once a satisfying part of these requirements are fulfilled for a standard rust project, new
languages will be included.

## Usage

> [!NOTE]  
> This is mostly a learning personal project and has no ambition to be suited for professional
> projects.

Release are not created yet for the project.

```sh
git clone git@github.com:arthur2klein/impact-scanner.git
cd impact-scanner
# Replace . with the path to your repository
cargo run -- --path="."
```
