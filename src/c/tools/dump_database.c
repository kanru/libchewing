/**
 * dump_database.c
 *
 * Copyright (c) 2014
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/**
 * @file dump_database.c
 *
 * @brief Dump system dictionary and phone phrase tree in a readable way.\n
 *
 *      This program reads in binary files of phone phrase tree\n
 * and dictionary generated by init_database.
 *      Output a human readable tree structure to stdout.\n
 */

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>

#ifdef HAVE_CONFIG_H
#    include <config.h>
#endif

#include "chewing-private.h"
#include "global-private.h"
#include "key2pho-private.h"
#include "memory-private.h"

#include "plat_types.h"
#include "private.h"

const char *dict = NULL;
const TreeType *root = NULL;
const char USAGE[] =
    "Usage: %s <data_directory>\n"
    "This program dumps the entire index structure to stdout.\n";

/*
 * node_pos: Index of the starting node. 0 represents the root.
 * indent: Degree of indentation.
 */
void dump(uint32_t node_pos, uint32_t indent)
{
    uint16_t key = 0;
    uint32_t i;

    for (i = 0; i < indent; i++)
        fputs("    ", stdout);

    key = GetUint16(root[node_pos].key);
    if (key != 0) {
        uint32_t beg = GetUint24(root[node_pos].child.begin);
        uint32_t end = GetUint24(root[node_pos].child.end);
        assert (beg < end);

        if (indent == 0)
            printf("count=%u,", key);
        else {
            char buf[MAX_UTF8_SIZE * BOPOMOFO_SIZE + 1];

            PhoneFromUint(buf, sizeof(buf), key);
            printf("key=%s,", buf);
        }
        printf(" begin=%u, end=%u\n", beg, end);

        for (i = beg; i < end; i++)
            dump(i, indent + 1);
    } else {
        uint32_t pos = GetUint24(root[node_pos].phrase.pos);
        uint32_t freq = GetUint24(root[node_pos].phrase.freq);

        printf("phrase=%s, freq=%u\n", &dict[pos], freq);
    }
}

void *read_input(const char *dir_name, const char *base_name, plat_mmap *mmap)
{
    char filename[PATH_MAX];
    size_t len;
    size_t offset;
    size_t file_size;
    size_t csize;
    void *buf = NULL;

    assert(dir_name);
    assert(base_name);

    len = snprintf(filename, sizeof(filename), "%s" PLAT_SEPARATOR "%s", dir_name, base_name);
    if (len + 1 > sizeof(filename)) {
        fprintf(stderr, "Too long path %s" PLAT_SEPARATOR "%s\n", dir_name, base_name);
        exit(-1);
    }

    file_size = plat_mmap_create(mmap, filename, FLAG_ATTRIBUTE_READ);
    if (file_size <= 0) {
        fprintf(stderr, "Error opening the file %s\n", filename);
        exit(-1);
    }

    offset = 0;
    csize = file_size;
    buf = plat_mmap_set_view(mmap, &offset, &csize);
    if (!buf) {
        fprintf(stderr, "Error reading the file %s\n", filename);
        exit(-1);
    }

    return buf;
}

int main(int argc, char *argv[])
{
    plat_mmap dict_mmap;
    plat_mmap tree_mmap;

    if (argc != 2) {
        printf(USAGE, argv[0]);
        return -1;
    }


    dict = (const char *) read_input(argv[1], DICT_FILE, &dict_mmap);
    root = (const TreeType *) read_input(argv[1], PHONE_TREE_FILE, &tree_mmap);

    dump(0, 0);

    plat_mmap_close(&dict_mmap);
    plat_mmap_close(&tree_mmap);

    return 0;
}