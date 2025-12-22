# sqlite-viz

A Rust CLI tool that parses SQLite database files and generates interactive HTML visualizations of the B-tree structure.

## Features

- **Direct SQLite parsing** - No libsqlite3 dependency, parses the file format directly
- **Interactive B-tree visualization** - Tree and force-directed graph views using D3.js
- **Page structure inspector** - Byte-level view of page contents with clickable cells
- **Cell content views** - Pretty, Hex dump, and ASCII representations
- **Filter by table/index** - Select specific B-trees to visualize
- **Overflow chain tracking** - Visual indication of overflow pages

## Installation

```bash
cargo build --release
```

The binary will be at `target/release/sqlite-viz`.

## Usage

### Generate visualization

```bash
sqlite-viz viz <DATABASE> [-o output.html]
```

Options:
- `-o, --output <FILE>` - Output HTML file (default: `<database>_viz.html`)

Example:
```bash
sqlite-viz viz mydb.sqlite -o visualization.html
```

### Show database info

```bash
sqlite-viz info <DATABASE> [-v]
```

Options:
- `-v, --verbose` - Show detailed information including all tables and indexes

### Dump to text file

Dump database structure to a human-readable text file for debugging:

```bash
sqlite-viz dump <DATABASE> [-o output.txt] [-t tree_name] [-p page_num]
```

Options:
- `-o, --output <FILE>` - Output file (default: `<database>.dump.txt`)
- `-t, --tree <NAME>` - Dump specific B-tree(s) by name (repeatable)
- `-p, --page <NUM>` - Dump specific page(s) by number (repeatable)

Examples:
```bash
# Dump entire database
sqlite-viz dump mydb.sqlite

# Dump only the users table
sqlite-viz dump mydb.sqlite -t users

# Dump specific pages
sqlite-viz dump mydb.sqlite -p 1 -p 2
```

The dump includes:
- Database header (page size, encoding, SQLite version, etc.)
- Schema overview with SQL definitions
- B-tree structure showing pages at each level
- Page details (header, cell pointers, free space)
- Cell contents with column types and values
- Hex+ASCII view of raw page bytes

## Visualization Features

### Main View

- **Sidebar** - Database info and schema list (tables/indexes)
- **Tree/Force view** - Toggle between hierarchical tree and force-directed graph
- **Page details panel** - Click any node to see page info and cells
- **Zoom controls** - Zoom in/out and reset

### Page Structure View

Click "View Page Structure" on any page to see:

- **Byte grid** - Visual representation of the entire page
  - Yellow: Page header
  - Orange: Cell pointers
  - Blue: Cell content
  - Red: Overflow cells
  - Gray: Free space
- **Cell labels** - Shows pointer order (P#) and physical order (→#)
- **Selection info** - Click cells or pointers for detailed info
- **Content tabs** - Pretty (parsed), Hex dump, ASCII views

### Color Legend

| Color | Page Type |
|-------|-----------|
| Blue | Interior Table |
| Green | Leaf Table |
| Purple | Interior Index |
| Teal | Leaf Index |
| Red | Overflow |

## SQLite Format Support

- Page sizes from 512 to 65536 bytes
- All B-tree page types (interior/leaf, table/index)
- Overflow page chains
- All serial types (NULL, integers, floats, blobs, text)
- UTF-8, UTF-16LE, UTF-16BE text encodings

## Other

This was built using Claude Code, and in part inspired by https://sqlite-internal.pages.dev/.

## License

MIT
