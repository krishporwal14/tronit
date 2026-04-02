# Tronit

A minimal yet feature-complete Git-like version control system written in Rust. Tronit implements core VCS functionality including object storage, index staging, branching, and safe working tree operations—designed as an educational tool to understand how modern version control systems work.

## Features

**Core VCS Mechanics:**

- **Content-Addressed Object Storage**: SHA-1 hashing with zlib compression for efficient storage
- **Index Staging**: BTreeMap-based staging with deduplication and replace-on-readd semantics
- **Commits**: Author name/email, timezone-aware timestamps, and commit messages
- **Branching**: Create, list, delete, and rename branches with symbolic HEAD references
- **History Traversal**: Walk commit ancestry with log output

**Advanced Features:**

- **Safe Checkout**: Dirty-worktree protection prevents accidental data loss when switching branches
- **Pathspec Support**: Glob patterns (`*`, `?`) for targeted file operations in `add` and `status` commands
- **Porcelain Output**: Machine-readable `status --porcelain` format for scripting and integration
- **Branch Lifecycle**: Full delete and rename operations with current-branch protection
- **Tree Objects**: Real Git-compatible binary tree entries with mode/name/hash serialization

## Installation

### Prerequisites

- Rust 1.70+ (see [rustup.rs](https://rustup.rs/) to install)

### Build from Source

```bash
git clone https://github.com/krishporwal14/tronit.git
cd tronit
cargo build --release
```

The binary will be at `target/release/tronit` (or `tronit.exe` on Windows).

### Run Directly

```bash
cargo run -- <command> [args]
```

## Quick Start

Initialize a new repository:

```bash
tronit init
```

Add files and create your first commit:

```bash
echo "Hello, Tronit!" > hello.txt
tronit add hello.txt
tronit commit -m "Initial commit" --author "Your Name <email@example.com>"
```

Check status:

```bash
tronit status
```

Create and switch branches:

```bash
tronit branch feature
tronit switch feature
tronit commit -m "Feature work" --author "Your Name <email@example.com>"
tronit switch main
```

## Commands

### `init`

Initialize a new Tronit repository in the current directory. Creates a `.tronit/` directory with required subdirectories.

```bash
tronit init
```

### `hash-object`

Compute SHA-1 hash of a file and optionally write it to the object database.

```bash
tronit hash-object FILE                # Show SHA-1 hash
tronit hash-object -w FILE             # Write object to database
```

### `cat-file`

Display object contents (commits, trees, or blobs).

```bash
tronit cat-file HASH                   # Display object
```

### `add`

Stage files for commit. Supports pathspec patterns to selectively stage files.

```bash
tronit add FILE                        # Stage a single file
tronit add src/                        # Stage all files in src/
tronit add "*.rs"                      # Stage all Rust files (glob pattern)
tronit add src/*.rs                    # Stage all .rs files in src/
tronit add FILE1 FILE2 "src/*"         # Multiple pathspecs
```

Pathspec matching supports:

- `*` – matches any sequence of characters (except `/` directory separator)
- `?` – matches any single character
- Literal paths

### `status`

Display the working tree status. Shows staged changes, unstaged changes, and untracked files.

**Human-readable format:**

```bash
tronit status
```

Output example:

```
On branch main
Staged:   hello.txt (added)
Unstaged: src/main.rs (modified)
Untracked: temp.log
```

**Porcelain format (for scripting):**

```bash
tronit status --porcelain [PATHSPEC...]
```

Output format:

```
## main
A  hello.txt
M  src/main.rs
_M src/utils.rs
?? temp.log
```

Porcelain codes:

- First column (staged): `A` (added), `M` (modified), `D` (deleted), ` ` (unchanged)
- Second column (unstaged): `M` (modified), `D` (deleted), ` ` (unchanged)
- Untracked files: `??`

Filter status by pathspec:

```bash
tronit status --porcelain "src/*"      # Only files matching src/*
tronit status --porcelain "*.rs"       # Only .rs files
```

### `commit`

Create a commit from staged changes with author metadata.

```bash
tronit commit -m "Commit message" --author "Name <email@example.com>"
```

Author format: `Name <email@example.com>` (required)

- Timestamps are automatically recorded in UTC with timezone offset

### `log`

Display commit history for the current branch.

```bash
tronit log [HASH]                      # Log from current HEAD
tronit log HASH                        # Log from specific commit
```

Output includes commit hash, author, date, and message.

### `branch`

Manage branches: list, create, delete, or rename.

```bash
tronit branch                          # List all branches (* marks current)
tronit branch BRANCH_NAME              # Create new branch from HEAD
tronit branch --delete BRANCH_NAME     # Delete a branch
tronit branch --move OLD_NAME NEW_NAME # Rename a branch
```

**Safety features:**

- Cannot delete the current branch
- Cannot rename to an existing branch name
- HEAD is updated automatically when renaming the current branch

### `switch`

Change the current branch with safe working tree checkout.

```bash
tronit switch BRANCH_NAME
```

**Safety checks:**

- Refuses to switch if the working tree has uncommitted changes (staged or unstaged)
- Automatically updates the index and working tree to match the target branch
- No data loss: either the switch succeeds completely, or it fails with no changes

Error example:

```
Error: working tree has local changes; commit/stash/clean before switching branches
```

## Architecture

### Object Database

Objects are stored in `.tronit/objects/` using SHA-1 content addressing:

- **Blobs**: Raw file contents
- **Trees**: Directory listings with (mode, name, hash) entries
- **Commits**: Metadata (parent, tree, author, message) + timestamp

All objects are zlib-compressed for space efficiency.

### Index

The staging area is stored in `.tronit/index` as a binary BTreeMap:

- Maps relative file paths → object hashes
- Supports atomic writes with deduplication
- Replace-on-readd: Re-adding a file replaces its previous entry

### References

Branch references stored in `.tronit/refs/heads/BRANCH_NAME`:

- Each file contains a commit hash
- Symbolic HEAD in `.tronit/HEAD` points to the current branch

### Working Tree

- Checked out from the commit tree on `switch`
- Untracked files are preserved
- Dirty-worktree check prevents unsafe branch switches

## Development

### Running Tests

```bash
cargo test
```

Test coverage includes:

- Object hashing and retrieval
- Index deduplication and staging semantics
- Pathspec matching with glob patterns
- Commit creation and log traversal
- Branch operations (create, delete, rename)
- Safe switch with dirty-worktree protection
- Porcelain status output format

### Project Structure

```
src/
  main.rs              # CLI routing and command dispatch
  lib.rs               # Module definitions
  object.rs            # Object hashing and storage
  repo.rs              # Repository metadata and refs
  index.rs             # Staging index
  ignore.rs            # .tronignore pattern matching
  pathspec.rs          # Glob pattern matching (add/status pathspecs)
  utils.rs             # Helper functions
  commands/
    init.rs            # Repository initialization
    hash_object.rs     # Object hashing
    cat_file.rs        # Object display
    add.rs             # File staging with pathspec support
    status.rs          # Working tree status (human + porcelain)
    commit.rs          # Commit creation
    log.rs             # Commit history
    branch.rs          # Branch management (create/list/delete/rename)
    switch.rs          # Branch switching with dirty-worktree protection
tests/
  project_tests.rs     # Integration tests
```

### Error Handling

All commands use `anyhow::Result<T>` for error handling with context-rich error messages:

```bash
tronit add nonexistent.txt
Error: failed to process file "nonexistent.txt": No such file or directory (os error 2)
```

## Design Decisions

1. **Content Addressing**: SHA-1 hashing ensures immutable, deduplicated storage
2. **Binary Tree Format**: Efficient, Git-compatible tree object serialization
3. **Pathspec Glob Patterns**: Intuitive file selection using standard wildcard syntax
4. **Porcelain Format**: Machine-readable output for tools and automation
5. **Dirty-Worktree Protection**: Prevents data loss from accidental overwrites
6. **Index Deduplication**: Efficient staging with automatic duplicate handling

## Limitations

- Single repository per directory (no nested submodules)
- No merge conflict resolution (fast-forward only)
- No stashing or rebasing
- Limited to local operation (no remote tracking)
- Pathspec support limited to `*` and `?` wildcards

## License

Educational project. Use freely for learning purposes.

## Contributing

Contributions welcome! Areas for improvement:

- Merge algorithms
- Rebase support
- Stash operations
- Remote tracking and push/pull
- More sophisticated pathspec patterns (glob, regex)
- Performance optimizations

## See Also

- [Git Book](https://git-scm.com/book/) – Authoritative Git documentation
- [Write Yourself a Git](https://jvns.ca/blog/2023/10/20/how-to-write-a-git-implementation-in-rust/) – Excellent Rust VCS guide
