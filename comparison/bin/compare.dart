/// Run uuid-format-tester encoders on every Appendix C UUID and dump
/// the results so they can be diffed against the printed draft cells
/// and against the vuuid_altenc crate.

import 'dart:typed_data';
import 'package:b/b.dart';
import 'package:base32/base32.dart' as b32;
import 'package:base32/encodings.dart' as b32e;

import 'package:comparison/encoders/base62id_uuid.dart';
import 'package:comparison/encoders/base85_uuid.dart';
import 'package:comparison/encoders/uuid_ncname.dart';
import 'package:comparison/constants/alphabets.dart';

const base16Lower = '0123456789abcdef';

const uuids = <String>[
  'f81d4fae-7dec-11d0-a765-00a0c91e6bf6', // C.1
  '00000000-0000-0000-0000-000000000000', // C.2
  'ffffffff-ffff-ffff-ffff-ffffffffffff', // C.3
  'c232ab00-9414-11ec-b3c8-9f6bdeced846', // C.4
  '000003e8-cbb9-21ea-b201-00045a86c8a1', // C.5
  '5df41881-3aed-3515-88a7-2f4a814cf09e', // C.6
  '919108f7-52d1-4320-9bac-f847db4148a8', // C.7
  '2ed6657d-e927-568b-95e1-2665a8aea6a2', // C.8
  '1ec9414c-232a-6b00-b3c8-9f6bdeced846', // C.9
  '017f22e2-79b0-7cc3-98c4-dc0c0c07398f', // C.10
  '2489e9ad-2ee2-8e00-8ec9-32d5f69181c0', // C.11a
  '5c146b14-3c52-8afd-938a-375d0df1fbf6', // C.11b
  '00000013-0000-0000-c000-000000000000', // C.12
];

Uint8List uuidBytes(String s) {
  final hex = s.replaceAll('-', '');
  final out = Uint8List(16);
  for (var i = 0; i < 16; i++) {
    out[i] = int.parse(hex.substring(i * 2, i * 2 + 2), radix: 16);
  }
  return out;
}

void main() {
  for (final u in uuids) {
    final bytes = uuidBytes(u);
    final hexNoDash = u.replaceAll('-', '');
    final b32Std = b32.base32.encode(bytes, encoding: b32e.Encoding.standardRFC4648).replaceAll('=', '');
    final b32Hex = b32.base32.encode(bytes, encoding: b32e.Encoding.base32Hex).replaceAll('=', '');
    final b32Crockford = b32.base32.encode(bytes, encoding: b32e.Encoding.crockford).replaceAll('=', '');
    final b36 = BaseConversion(from: base16Lower, to: '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ')(hexNoDash);
    final b52 = BaseConversion(from: base16Lower, to: base52Alphabet)(hexNoDash);
    final b58btc = BaseConversion(from: base16Lower, to: '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz')(hexNoDash);
    final b62Sort = BaseConversion(from: base16Lower, to: base62SortAlphabet)(hexNoDash);
    final b62Ieee = BaseConversion(from: base16Lower, to: base62IeeeAlphabet)(hexNoDash);
    final b64Std = _b64(bytes, '+/');
    final b64Url = _b64(bytes, '-_');
    final z85 = encodeZ85(bytes);
    final ascii85 = encodeAscii85(bytes);
    final btoa = encodeBtoa(bytes);
    final zmodem = encodeZmodem(bytes);
    final nc32 = UuidNCName.encodeBase32(bytes);
    final nc58 = UuidNCName.encodeBase58(bytes);
    final nc64 = UuidNCName.encodeBase64(bytes);
    final b62id = encodeBase62Id(u);

    print('UUID $u');
    print('  b32        $b32Std');
    print('  b32hex     $b32Hex');
    print('  crockford  $b32Crockford');
    print('  b36        $b36');
    print('  b52        $b52');
    print('  b58btc     $b58btc');
    print('  b62sort    $b62Sort');
    print('  b62ieee    $b62Ieee');
    print('  b64std     $b64Std');
    print('  b64url     $b64Url');
    print('  z85        $z85');
    print('  ascii85    $ascii85');
    print('  btoa       $btoa');
    print('  zmodem     $zmodem');
    print('  nc32       $nc32');
    print('  nc58       $nc58');
    print('  nc64       $nc64');
    print('  b62id      $b62id');
  }
}

String _b64(Uint8List bytes, String tail) {
  // RFC 4648 Base64 with the configurable last two characters.
  final alpha = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789' + tail;
  final n = (bytes.length * 8 + 5) ~/ 6;
  var acc = 0;
  var nacc = 0;
  final out = StringBuffer();
  var i = 0;
  for (var k = 0; k < n; k++) {
    while (nacc < 6) {
      acc = (acc << 8) | (i < bytes.length ? bytes[i] : 0);
      nacc += 8;
      if (i < bytes.length) i++;
    }
    nacc -= 6;
    out.write(alpha[(acc >> nacc) & 0x3f]);
    acc &= (1 << nacc) - 1;
  }
  return out.toString();
}
