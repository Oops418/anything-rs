# Anything-rs

This project is currently under active development in early development stage.

## How it works

![How it works](resources/how-it-works.png)

## TODO List

The following items are currently on our development roadmap:

- [ ] Fix menu name error
- [ ] Fix cols sorting
- [ ] Improve UI styling
- [ ] Adjust UI color scheme(Nord Light theme in VSCode)
- [ ] Fix index case sensitivity
- [ ] Add Linux support
- [ ] Add cache for search to avoid repeat searches
- [ ] Add UI for custom included/excluded folders
- [ ] Improve tokenizer
- [ ] Improve search logic
- [ ] Improve performance

known issues:
[Duplicated filesystem events](https://github.com/notify-rs/notify/issues/272) seem to be caused by the FSEvents API on macOS.
