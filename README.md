# Rusty Tree
## Summary
This is a very simple tool to render a file tree in the terminal. It's a small personal project to familiarize myself with rust's json parsing.

**This requires a [NerdFont](https://www.nerdfonts.com) to render glyphs properly.**

Example usages:
```
> rusty-tree.exe
> rusty-tree.exe ./src
```

![Showcase](/docs/Showcase.jpg)

## Configuration
Data folder:
- `glyphs.json`: dictionary of icon keys to glyphs. Make sure to use an editor with a Nerd Font.
- `colors.json`: maps filetypes to colors
- `icons.json`: maps filetypes to icon keys
- `settings.json`: lists rules to ignore subtrees