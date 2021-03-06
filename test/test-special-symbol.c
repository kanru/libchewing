/**
 * test-special-symbol.c
 *
 * Copyright (c) 2012
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifdef HAVE_CONFIG_H
#include <config.h>
#endif

#include <stdlib.h>
#include <string.h>

#include "chewing.h"
#include "test.h"

static const TestData SPECIAL_SYMBOL_TABLE[] = {
	{ .token = "[", .expected = "「" },
	{ .token = "]", .expected = "」" },
	{ .token = "{", .expected = "『" },
	{ .token = "}", .expected = "』"},
	{ .token = "'", .expected = "、" },
	{ .token = "<<>", .expected = "，" },
	{ .token = ":", .expected = "：" },
	{ .token = "\"", .expected = "；" },
	{ .token = ">", .expected = "。" },
	{ .token = "~", .expected = "～" },
	{ .token = "!", .expected = "！" },
	{ .token = "@", .expected = "＠" },
	{ .token = "#", .expected = "＃" },
	{ .token = "$", .expected = "＄" },
	{ .token = "%", .expected = "％" },
	{ .token = "^", .expected = "︿" },
	{ .token = "&", .expected = "＆" },
	{ .token = "*", .expected = "＊" },
	{ .token = "(", .expected = "（" },
	{ .token = ")", .expected = "）" },
	{ .token = "_", .expected = "﹍" },
	{ .token = "+", .expected = "＋" },
	{ .token = "=", .expected = "＝" },
	{ .token = "\\", .expected = "＼" },
	{ .token = "|", .expected = "｜" },
	{ .token = "?", .expected = "？" },
	{ .token = ",", .expected = "，" },
	{ .token = ".", .expected = "。" },
	{ .token = ";", .expected = "；" },
};

int is_bopomofo_collision_key( const char *key )
{
	static const char *COLLISION_KEY[] = {
		"<<>",
		">",
		";",
		",",
		".",
	};

	for ( int i = 0; i < ARRAY_SIZE( COLLISION_KEY ); ++i ) {
		if ( strcmp( key, COLLISION_KEY[i] ) == 0 ) {
			return 1;
		}
	}
	return 0;
}

void test_in_chinese_mode()
{
	chewing_Init( NULL, NULL );

	ChewingContext *ctx = chewing_new();
	ok( ctx, "chewing_new shall not return NULL" );

	chewing_set_maxChiSymbolLen( ctx, 16 );

	for ( int i = 0; i < ARRAY_SIZE( SPECIAL_SYMBOL_TABLE ); ++i ) {
		// If bopomofo symbol is collided with special symbol, use
		// bopomofo symbol
		if ( is_bopomofo_collision_key( SPECIAL_SYMBOL_TABLE[i].token ) )
			continue;

		type_keystoke_by_string( ctx, SPECIAL_SYMBOL_TABLE[i].token );
		type_keystoke_by_string( ctx, "<E>" );
		ok_commit_buffer( ctx, SPECIAL_SYMBOL_TABLE[i].expected );
	}

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_in_easy_symbol_mode()
{
	chewing_Init( NULL, NULL );

	ChewingContext *ctx = chewing_new();
	ok( ctx, "chewing_new shall not return NULL" );

	chewing_set_maxChiSymbolLen( ctx, 16 );
	chewing_set_easySymbolInput( ctx, 1 );

	for ( int i = 0; i < ARRAY_SIZE( SPECIAL_SYMBOL_TABLE ); ++i ) {
		type_keystoke_by_string( ctx, SPECIAL_SYMBOL_TABLE[i].token );
		type_keystoke_by_string( ctx, "<E>" );
		ok_commit_buffer( ctx, SPECIAL_SYMBOL_TABLE[i].expected );
	}

	chewing_delete( ctx );
	chewing_Terminate();
}

int is_fullshape_collision_key( const char *key )
{
	static const char *COLLISION_KEY[] = {
		"\"",
		"'",
		"/",
		"<<>",
		">",
		"`",
		"[",
		"]",
		"{",
		"}",
		"+",
		"-",
	};

	for ( int i = 0; i < ARRAY_SIZE( COLLISION_KEY ); ++i ) {
		if ( strcmp( key, COLLISION_KEY[i] ) == 0 ) {
			return 1;
		}
	}
	return 0;
}

void test_in_fullshape_mode()
{
	chewing_Init( NULL, NULL );

	ChewingContext *ctx = chewing_new();
	ok( ctx, "chewing_new shall not return NULL" );

	chewing_set_maxChiSymbolLen( ctx, 16 );
	chewing_set_ChiEngMode( ctx, SYMBOL_MODE );
	chewing_set_ShapeMode( ctx, FULLSHAPE_MODE );

	for ( int i = 0; i < ARRAY_SIZE( SPECIAL_SYMBOL_TABLE ); ++i ) {
		// If fullshape symbol is collided with special symbol, use
		// fullshape symbol
		if ( is_fullshape_collision_key( SPECIAL_SYMBOL_TABLE[i].token ) )
			continue;

		type_keystoke_by_string( ctx, SPECIAL_SYMBOL_TABLE[i].token );
		ok_commit_buffer( ctx, SPECIAL_SYMBOL_TABLE[i].expected );
	}

	chewing_delete( ctx );
	chewing_Terminate();
}

int main()
{
	putenv( "CHEWING_PATH=" CHEWING_DATA_PREFIX );
	putenv( "CHEWING_USER_PATH=" TEST_HASH_DIR );

	test_in_chinese_mode();
	test_in_easy_symbol_mode();
	test_in_fullshape_mode();

	return exit_status();
}
