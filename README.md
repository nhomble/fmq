# another query cli for frontmatter

help my agents programmatically manage properties of my markdown files

## Implementation

1. Stream the input file for the frontmatter section
2. Manipulate the frontmatter with `yq`
3. Then output the query passed into `yq`

In other words, I don't expect this to be a complicated script beyond frontmatter parsing. The rest can be delegated to `yq` or an equivalent library
if it accepts a familiar syntax. 

Write this is simple rust for me to understand/learn and get the cross platform cli benefits.
