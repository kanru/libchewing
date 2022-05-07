/*
 * Copyright (c) 2022
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifndef chewing_internal_bindings_h
#define chewing_internal_bindings_h

#pragma once

/* Warning, this file is autogenerated by cbindgen. Don't modify this manually. */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum KeyBehavior {
  Ignore = 0,
  Absorb,
  Commit,
  KeyError,
  Error,
  NoWord,
  OpenSymbolTable,
} KeyBehavior;

typedef enum KeyboardLayoutCompat {
  Default = 0,
  Hsu,
  Ibm,
  GinYieh,
  Et,
  Et26,
  Dvorak,
  DvorakHsu,
  DachenCp26,
  HanyuPinyin,
  ThlPinyin,
  Mps2Pinyin,
  Carpalx,
} KeyboardLayoutCompat;

void *NewPhoneticEditor(enum KeyboardLayoutCompat kb_type);

void FreePhoneticEditor(void *editor_keymap_ptr);

enum KeyBehavior PhoneticEditorInput(void *editor_keymap_ptr, int32_t key);

void PhoneticEditorSyllable(void *editor_keymap_ptr, int32_t *pho_inx);

void PhoneticEditorSyllableAlt(void *editor_keymap_ptr, int32_t *pho_inx);

void PhoneticEditorKeyseq(void *editor_keymap_ptr, char *key_seq);

uint16_t PhoneticEditorSyllableIndex(void *editor_keymap_ptr);

uint16_t PhoneticEditorSyllableIndexAlt(void *editor_keymap_ptr);

void PhoneticEditorRemoveLast(void *editor_keymap_ptr);

void PhoneticEditorRemoveAll(void *editor_keymap_ptr);

int32_t PhoneticEditorKbType(void *editor_keymap_ptr);

bool PhoneticEditorIsEntering(void *editor_keymap_ptr);

#endif /* chewing_internal_bindings_h */