testdir = $(top_builddir)/test
TESTDATA = $(testdir)/test.txt
DEFAULT_TESTDATA = $(testdir)/default-test.txt
LIBCHEWING = $(top_builddir)/src/libchewing.la
DATADIR = $(top_builddir)/data
CHEWINGDATA = \
	$(DATADIR)/us_freq.dat \
	$(DATADIR)/ch_index.dat \
	$(DATADIR)/dict.dat \
	$(DATADIR)/ph_index.dat \
	$(DATADIR)/fonetree.dat

$(TESTDATA): $(DEFAULT_TESTDATA)
	if [ ! -f $(TESTDATA) ]; then \
		cp $(DEFAULT_TESTDATA) $(TESTDATA) ; \
	fi

check: testchewing $(TESTDATA) $(CHEWINGDATA)
	$(testdir)/testchewing < $(TESTDATA)

vcheck: check
	valgrind \
		--leak-check=full \
		--show-reachable=yes \
		$(testdir)/.libs/testchewing < $(TESTDATA)

bin_PROGRAMS = testchewing genkeystroke do-test

AM_CFLAGS = \
	-I$(top_builddir)/include \
	-DCHEWING_DATA_PREFIX="\"$(DATADIR)\"" \
	-DTEST_HASH_DIR="\"$(PWD)\""

testchewing_SOURCES = testchewing.c
testchewing_LDADD = $(LIBCHEWING) -lncursesw
testchewing_DEPENDENCIES = $(LIBCHEWING)

genkeystroke_SOURCES = gen_keystroke.c
genkeystroke_LDADD = $(LIBCHEWING) -lncursesw
genkeystroke_DEPENDENCIES = $(LIBCHEWING)

test_case_SRC = test-utf8.c test-key2pho.c
do_test_SOURCES = do-test.c $(test_case_SRC)
do_test_LDADD = $(LIBCHEWING) -lcheck
do_test_DEPENDENCIES = $(LIBCHEWING)