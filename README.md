ipassgen
=======

Entropy-requirement based password generator.

## Usage

Char-based mode:
```
$ ipassgen -aA0 -E 80.0
BIrfarGdBN666q
```

Word-based mode (Diceware):
```
$ ipassgen -m diceware -E 80.0
jh dang front red von wack y's
```

## Options

### Mode: `-m <MODE>`

`ipassgen` supports character-based password and word-based passphrase.

| Mode | Type | Description |
|:-:|:-:|:--|
| chars | Chars | Character-based password |
| basic-words | Words | Elementary words list (1358 words) |
| diceware | Words | Original [Diceware](http://world.std.com/~reinhold/diceware.html) (7776 words) |
| diceware-alnum | Words | Alpha-numeric words from Diceware (7697 words) |

### Character set specifier: `-a`, `-A`, `-0`, `-!`, (Chars mode only)

Specify character set.

| Option | Set |
|:-:|:--|
| a | Lower case, a-z |
| A | Upper case, A-Z |
| 0 | Numbers, 0-9 |
| ! | ASCII symbols |

### Specify symbols: `-s <SYMBOLS>`

If you wanto to specify symbols to use, use `-s` option.
```
$ ipassgen -aA0 -s '%&()*+,-./:;<=>?@[]^_|~'
r(MY3lv5X.7Q
```


### Entropy: `-E <ENTROPY>` / `--entropy <ENTROPY>`

Specify target entropy in bits.

### Length: `-L <LEN>` / `--length <LEN>`

Specify output length.

In char-based mode, output LEN characters. \
In word-based mode, output LEN words.

If you specify short length that is not enough to generate default entropy target, you must spceify smaller entropy target (`-E`).

### Maximum output length: `-M <BYTES>` / `--max-length <BYTES>`

Specify maximum output length in bytes.

Default value is 72 bytes, from bcrypt limitation.

### Separator: `-S <SEP>` / `--sep <SEP>`

Specify word separator.
Default is a space.

## License

`ipassgen` is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT).