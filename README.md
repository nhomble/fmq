# fmq

jq for markdown frontmatter.

Query and mutate YAML frontmatter in markdown files using jq syntax.

## Install

### From releases

Download the latest binary from [GitHub Releases](https://github.com/nhomble/fmq/releases).

### From source

```bash
cargo install --path .
```

## Usage

```bash
fmq <expression> [file]
```

- If `file` is omitted, reads from stdin
- Queries output the result to stdout
- Mutations output the modified markdown to stdout

## Examples

### Query

```bash
# Get a field
fmq '.title' post.md
# Hello World

# Get nested field
fmq '.author.name' post.md
# Alice

# Get array
fmq '.tags' post.md
# ["rust", "cli"]

# From stdin
cat post.md | fmq '.title'
```

### Mutate

```bash
# Set a field
fmq '.title = "New Title"' post.md > updated.md

# Add to array
fmq '.tags += ["new-tag"]' post.md > updated.md

# Delete a field
fmq 'del(.draft)' post.md > updated.md

# Update nested field
fmq '.author.name = "Bob"' post.md > updated.md
```

### Piping

```bash
# Query then mutate
fmq '.tags' post.md | jq '.[0]'

# Chain mutations
fmq '.title = "New"' post.md | fmq '.draft = false' > updated.md
```
