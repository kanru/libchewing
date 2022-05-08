/**
 * dict-private.h
 *
 * Copyright (c) 2008
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/* *INDENT-OFF* */
#ifndef _CHEWING_DICT_PRIVATE_H
#define _CHEWING_DICT_PRIVATE_H
/* *INDENT-ON* */

#include "chewing-private.h"

#ifndef SEEK_SET
#define SEEK_SET 0
#endif

#ifndef HAVE_RUST

#define PHONE_PHRASE_NUM (162244)

int GetCharFirst(ChewingData *, Phrase *, uint16_t);
int InitDict(ChewingData *pgdata, const char *prefix);
void TerminateDict(ChewingData *pgdata);
int GetPhraseFirst(ChewingData *pgdata, Phrase *phr_ptr, const TreeType *phrase_parent);
int GetVocabNext(ChewingData *pgdata, Phrase *phr_ptr);

#endif

/* *INDENT-OFF* */
#endif
/* *INDENT-ON* */
