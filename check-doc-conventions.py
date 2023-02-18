#! /usr/bin/env python3
import argparse
import os
from abc import ABC, abstractmethod
from enum import Enum, auto
from typing import List

parser = argparse.ArgumentParser(
    description="Checks that Rust doc comments conform to the established conventions for the project.")
parser.add_argument("--lints", "-l", action="store_true",
                    help="just list the lints with descriptions")
args = parser.parse_args()


class DoctestBlock(Enum):
    NONE = auto()
    BEGIN = auto()
    CODE = auto()
    END = auto()


class RustLine:
    """
    A line of a Rust source file.

    Attributes:
    doc_comment - Whether this line is a doc comment (bool)
    content - The content of the line after the doc comment if applicable
    doctest_block - What type of doctest line this is (DoctestBlock)
    """

    def __init__(self, raw_line: str, in_doctest: bool):
        """
        Parse a raw line from a file.
        """
        sline = raw_line.strip()
        if sline.startswith("///") or sline.startswith("//!"):
            # A doc comment line
            self.doc_comment = True
            self.content = sline[3:].strip()
            if self.content.startswith("```"):
                if in_doctest:
                    self.doctest_block = DoctestBlock.END
                else:
                    self.doctest_block = DoctestBlock.BEGIN
            else:
                if in_doctest:
                    self.doctest_block = DoctestBlock.CODE
                else:
                    self.doctest_block = DoctestBlock.NONE
        else:
            # A normal line of code
            self.doc_comment = False
            self.content = sline
            self.doctest_block = DoctestBlock.NONE

    def format(self) -> str:
        """
        Returns the line for printing.
        """
        if self.doc_comment:
            return "/// " + self.content
        else:
            return self.content


class SourceFile:
    """
    A Rust source file.

    Attributes:
    file_path - Path to this source file (str)
    lines - List of lines ([RustLine])
    """

    def __init__(self, file_path: str):
        """
        Read and parse the file from a path.
        """
        lines = []
        with open(file_path, "r") as f:
            in_doctest = False
            for line in f:
                rust_line = RustLine(line, in_doctest)

                # Did we enter a doctest block?
                if rust_line.doctest_block == DoctestBlock.BEGIN:
                    in_doctest = True
                elif rust_line.doctest_block == DoctestBlock.END:
                    in_doctest = False

                lines.append(rust_line)

        self.file_path = file_path
        self.lines = lines

    def format_line(self, line_num: int):
        """
        Print the line with the line number.
        """
        return self.file_path + ":" + str(line_num + 1) + " " + self.lines[line_num].format()


class Lint:
    """
    A general doc comment lint.

    Attributes:
    name - The name of this lint (str)
    description - A description of this lint (str)
    """

    @abstractmethod
    def __init__(self):
        pass

    def alert(self, source_file: SourceFile, line_num):
        """
        Prints the lint alert
        """
        print(self.name + ": " + source_file.format_line(line_num))

    def describe(self):
        """
        Prints the name and description of the lint.
        """
        print(self.name + ":", self.description)

    @abstractmethod
    def check_file(self, source_file: SourceFile):
        """
        Check that a file conforms to the lint and print out any non-conforming lines.
        """
        pass


class IntroLint(Lint):
    """
    Verifies that the intro of every doc comment is only a single
    sentence.
    """

    def __init__(self):
        self.name = "intro_sentence"
        self.description = "The introduction to every doc comment must be a single isolated sentence"

    def check_file(self, source_file: SourceFile):
        lines = source_file.lines

        for ln, line in enumerate(lines):
            # First check for multiple sentences in the doc introduction
            if line.doc_comment and (ln == 0 or not lines[ln-1].doc_comment):
                # Append the following lines until a blank or non-comment line is reached
                long_line = line.content
                for next_line in lines[ln + 1:]:
                    if not next_line.doc_comment or len(next_line.content) == 0:
                        break

                    long_line += " " + next_line.content

                # Determine if the full intro text is more than one sentence
                for part in long_line.split(". ")[1:]:
                    # Is the first letter of the next "sentence" a capital letter?
                    # This will not be the case for abbreviations like "i.e.", and these
                    # are perfectly ok since they do not start a new sentence.
                    if part[0].isupper():
                        self.alert(source_file, ln)
                        break


class CrossRefLint(Lint):
    """
    Verifies that cross references all use code font.
    """

    def __init__(self):
        self.name = "cross_ref_code"
        self.description = "All cross references must use the Markdown code font"

    def check_file(self, source_file: SourceFile):
        lines = source_file.lines

        for ln, line in enumerate(lines):
            if line.doc_comment and line.doctest_block == DoctestBlock.NONE:
                line_left = line.content
                while True:
                    idx = line_left.find("[")
                    if idx > -1:
                        # Found a cross reference
                        line_left = line_left[idx+1:]
                        split = line_left.split("]")
                        if len(split) == 0 or not split[1].startswith("("):
                            # Okay this is not an ordinary link.
                            if not split[0].startswith("`"):
                                self.alert(source_file, ln)
                    else:
                        break


class IntroVerbLint(Lint):
    """
    Certain item intro sentences must start with an action verb in the proper tense.
    """

    def __init__(self):
        self.name = "intro_verb"
        self.description = "Certain (TODO) intro sentences must begin with an action verb describing what it does, e.g. 'Returns X' instead of 'Return X'"

    def check_file(self, source_file: SourceFile):
        pass


# All our lints.
lints = [IntroLint(), CrossRefLint(), IntroVerbLint()]


def source_files() -> str:
    """
    Generator over al Rust source file paths.
    """
    src_dirs = ("src", os.path.join("aoc_derive", "src"))

    for src_dir in src_dirs:
        for (dir, tmp, fnames) in os.walk(src_dir):
            for fname in fnames:
                if fname.endswith(".rs"):
                    yield os.path.join(dir, fname)


if args.lints:
    for lint in lints:
        lint.describe()
else:
    # Check every file for every lint
    for source_path in ["src/aoc/grid.rs"]:
        # for source_path in source_files():
        source_file = SourceFile(source_path)

        for lint in lints:
            lint.check_file(source_file)
