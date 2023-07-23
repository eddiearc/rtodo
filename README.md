# rtodo
a simple todo-list program written by rust

for rust practice


Print help command:
```bash
rtodo -h
```
will display:
```bash
a simple todo-list program written by rust

Usage: rtodo [OPTIONS]

Options:
  -a, --add <ADD>        Add a todo item
  -l, --list <LIST>      list add todo items, if present fuzzy match item content [default: ]
  -d, --delete <DELETE>  delete todo item by item NO
  -h, --help             Print help
  -V, --version          Print version
```

Add a todo item:
```bash
rtodo -a 'learn rust for a hour today'
```
will display:
```bash
NO:1 -> learn rust for a hour today
```

Remove a todo item:
```bash
rtodo -a 1
```
will display nothing.