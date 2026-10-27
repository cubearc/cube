# Decode symbolic clues

Choose an explicit symbol alphabet, supplied book or grid traversal. Cube reports the chosen interpretation rather than inferring a hidden encoding.

## CLI

### Interpret directional symbols

```sh
cube number clock --time 3:00 --decode --json
cube number clock --hour-millidegrees -45000 --minute-millidegrees 0 --json
cube number clock --time 12:45 --json
```


```sh
cube encode semaphore --text 'A B'
cube decode semaphore --text 'S,SW / S,W'
```


### Extract from a caller-supplied book

```sh
printf 'Cube Uses Binary Evidence' > /tmp/cube-book-source.txt
cube cipher book --source /tmp/cube-book-source.txt --text '1:1,2:1,3:1,4:1'
# CUBE
```


### Arrange text in scalar grids

```sh
cube text grid --text ABCDEF --json
cube text grid --text ABCDEF --width 3
cube text grid --text 0101 --width 2 --colors '[["0","000000ff"],["1","ffffffff"]]' --output grid.png
```


### Generate string combinations

```sh
cube text combinations --text '[["a","b"],["1","2"]]' --separator - --plan
cube text combinations --text '[["a","b"],["1","2"]]' --separator -
```


## Python

### Work with grids, combinations and directional symbols

Use the linked reference for the supported model and its examples.

## Reference

- [Symbolic encodings](../reference/topics/symbolic-encodings.md)
- [Text operations](../reference/topics/text-operations.md)
