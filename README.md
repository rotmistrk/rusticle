# rusticle

A Tcl-compatible scripting language interpreter written in pure Rust.

## Features

- **Tcl-compatible syntax** — braces, quotes, command substitution, variable substitution
- **Modern enhancements** — lexical scoping, typed declarations, structured literals, accessor syntax
- **Embeddable** — use as a library with custom commands, or run standalone
- **Pure Rust** — zero dependencies, no unsafe code
- **Static analysis** — load-time validation catches errors before execution

## Quick Start

```bash
# Run a script
cargo run -- examples/basics.tcl

# Interactive REPL
cargo run
```

## Usage as a Library

```rust
use rusticle::interpreter::Interpreter;

let mut interp = Interpreter::new();
interp.eval("set x 42").unwrap();
let result = interp.eval("expr {$x + 8}").unwrap();
assert_eq!(result.as_str(), "50");
```

### Custom Commands

```rust
use rusticle::interpreter::Interpreter;
use rusticle::value::Value;
use rusticle::error::TclError;

let mut interp = Interpreter::new();
interp.register_command("greet", |_interp, args| {
    if args.len() != 2 {
        return Err(TclError::new("usage: greet name"));
    }
    Ok(Value::from(format!("Hello, {}!", args[1].as_str())))
});
```

## Supported Tcl Commands

### Control Flow
`if`, `while`, `for`, `foreach`, `switch`, `break`, `continue`, `return`, `error`, `catch`, `try`

### Variables & Procedures
`set`, `unset`, `incr`, `append`, `proc`, `apply`, `uplevel`, `upvar`, `global`, `variable`

### Strings
`string length`, `string index`, `string range`, `string match`, `string map`, `string trim`, `string tolower`, `string toupper`, `string repeat`, `string replace`, `string first`, `string last`, `string is`, `string cat`, `string reverse`, `format`, `scan`

### Lists
`list`, `lindex`, `llength`, `lrange`, `lappend`, `linsert`, `lreplace`, `lsearch`, `lsort`, `lrepeat`, `lreverse`, `lmap`, `lfilter`, `lreduce`, `join`, `split`, `concat`

### Dictionaries
`dict create`, `dict get`, `dict set`, `dict unset`, `dict exists`, `dict keys`, `dict values`, `dict size`, `dict for`, `dict map`, `dict filter`, `dict merge`, `dict append`, `dict incr`, `dict lappend`, `dict remove`, `dict replace`, `dict with`

### Expressions
`expr` with full arithmetic, comparison, logical, and string operators

### I/O
`puts`, `gets`

### Introspection
`info exists`, `info commands`, `info procs`, `info vars`, `info locals`, `info globals`, `info args`, `info body`, `info level`

## Contexts

Contexts are named scopes for structured application state. Variables are
accessed as `$ctx::var` and are globally visible — readable and writable from
any scope, including inside procs.

```tcl
context app {
    declare mode : enum {normal insert}
    set mode normal
    set count 0
}

# Read from anywhere
puts "Mode is $app::mode"

# Write from a proc — targets global scope automatically
proc increment {} {
    incr app::count
}

# Type-safe: invalid values are rejected
set app::mode insert   ;# ok
set app::mode bogus    ;# ERROR: not a valid value for enum {normal insert}
```

**Key behaviors:**
- `set ctx::var value` always writes to global scope (where contexts live)
- Writing to an undefined context fails: `can't set "bad::x": no such context "bad"`
- Type declarations (`declare var : type`) are enforced on every assignment
- The validator warns at compile-time about references to undefined contexts

Supported types for `declare`: `string`, `int`, `float`, `bool`, `list`, `dict`, `enum {values...}`. Append `?` for nullable (e.g., `int?`).

## Examples

See the `examples/` directory:
- `basics.tcl` — variables, control flow, procedures
- `data_structures.tcl` — lists and dictionaries
- `functional.tcl` — higher-order functions, lmap, lfilter, lreduce
- `contexts.tcl` — lexical scoping and contexts

## License

MIT
