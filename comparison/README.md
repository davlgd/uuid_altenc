# Cross-check harness

Pure-Dart project that drives the encoders from
[`uuid-format-tester`](https://github.com/daegalus/uuid-format-tester)
on every Appendix C UUID, so their output can be diffed against this
crate (`uuid_altenc`) and against the printed draft cells.

## Run

```sh
# install Dart 3.5+ (e.g. via mise: `mise use dart`)
cd comparison
dart pub get
dart run bin/compare.dart
```

The encoder source files in `lib/encoders/` and `lib/constants/` are
copies of the Apache-licensed sources from `uuid-format-tester`,
reproduced verbatim so the project can be built without the Flutter
SDK.
