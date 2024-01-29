# Language detection

In order to classify posts better, Kitsune attempts to automatically guess the language a post is written in to improve the search experience by using language-specific tokenization.

It can do that through a number of language detection backends.  
The currently supported backends are:

- `none`: Use no detection and just return the default language
- `whatlang`: Use the [whatlang](https://github.com/greyblake/whatlang-rs) library (recommended)
- `whichlang`: Use the [whichlang](https://github.com/quickwit-oss/whichlang) library

### Note on the backends

In general you should prefer the `whatlang` backend as it offers reliability calculations and covers a wide range of languages.

`whichlang` is generally _faster_ than `whatlang` but has less supported languages and doesn't offer any reliability calculations, meaning the classifications might be _way off_
and won't ever fall back on the default language.

You probably don't want to use the `none` backend unless you are 100% confident that the language detection is too resource intensive for your installation (which is extremely unlikely!)

## Configuration

### `backend`

In order to set the backend, choose one of the above mentioned supported backends.  
It is configured like so:

```toml
[language-detection]
backend = "whatlang" # Use "whatlang" to detect the language
default-language = "en"
```

### `default-language`

This setting sets the default language Kitsune falls back onto.  
If the language couldn't be guessed for whatever reason (be it an internal failure of the backend or because the reliability of the guess wasn't given),
this is the language Kitsune returns.

You might want to adjust this if you know the language that will be predominantly posted in on your instance to improve search quality, especially on shorter posts.

The setting accepts any valid ISO 639-1 or 639-3 code.

ISO 639-1:

```toml
[language-detection]
backend = "whatlang"
default-language = "de" # Use German to fall back on by referencing its ISO 639-1 value
```

ISO 639-3:

```toml
[language-detection]
backend = "whatlang"
default-language = "kor" # Use Korean to fall back on by referencing its ISO 639-3 value
```
