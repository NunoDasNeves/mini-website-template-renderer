# Website template rendeerererrer

Takes a directory containing some basic templates, md files and other files, and produces another directory containing static html files.
Useful for a simple personal website with static pages + a blog.

## Directory structure

This program expects an `in_dir` containing:
* `template.html` - a html file with a section for md files to be rendered, denoted with {{content}}
* `blog.html` - a template for a blog tile, to be rendered in a list of blogs in `out_dir/blogs.html`
* `blogs` - a directory containing `.md` files which are interpreted as blogs
* `*.md` files - files that will be rendered as html and inserted into `template.html`
* other files and directories will be copied as-is to the output directory

## Example
There is an example directory with a basic website located in in\_dir. To render it, use:
```renderer in_dir out_dir```

