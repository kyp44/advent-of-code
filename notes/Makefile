###############################################################################
# FILE: Makefile
#
# Author: Dan Whitman
##############################################################################
TARGET = notes

.PHONY: all clean

#==============================================================================
all:
#==============================================================================
	pdflatex -halt-on-error $(TARGET).tex

#==============================================================================
clean:
#==============================================================================
	rm -f *.aux *.log *.pdf *.out *.toc
	rm -f -r *.png figs/
	rm -f -r __pycache__
