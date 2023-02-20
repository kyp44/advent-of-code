#! /usr/bin/env python3
"""
Run with `-h` for more info.

NOTE: Looked into creating lints in Rust itself using `dylint`, but this seems overly complex
for what I need, and it is also geared to towards actual code lints rather than doc comments,
like `clippy`. I was also unable to find a Rust parser module for Python, only getting results
for the other way around.
"""
import argparse
import os
from abc import ABC, abstractmethod
from enum import Enum, auto
from typing import List, Tuple

parser = argparse.ArgumentParser(
    description="Checks that Rust doc comments conform to the established conventions for the project.")
parser.add_argument("--lints", "-l", action="store_true",
                    help="just list the lints with descriptions")
args = parser.parse_args()


class DocLineType(Enum):
    NONE = auto()
    TEXT = auto()
    TEXT_HEADER = auto()
    TEXT_EMPTY = auto()
    CODE_BEGIN = auto()
    CODE = auto()
    CODE_END = auto()


class ItemType(Enum):
    NONE = auto()
    FUNCTION = auto()
    STRUCT = auto()
    ENUM = auto()
    TRAIT = auto()
    TYPE = auto()
    IMPL = auto()
    USE = auto()


class RustLine:
    """
    A line of a Rust source file.

    Attributes:
    doc_line_type - Whether this line is doc command and which type if so (DocLineType)
    comment - Whether this line is a normal comment (bool)
    content - The content of the line after the comment if applicable (str)
    item_type - If an item definition, the type of the item (ItemType)
    """

    def __init__(self, raw_line: str, in_doctest: bool):
        """
        Parse a raw line from a file.
        """
        sline = raw_line.strip()

        self.doc_line_type = DocLineType.NONE
        self.comment = False
        self.content = sline
        self.item_type = ItemType.NONE

        if sline.startswith("///") or sline.startswith("//!"):
            # A doc comment line
            self.doc_line_type = DocLineType.TEXT
            self.comment = True
            self.content = sline[3:].strip()
            if self.content.startswith("#"):
                self.doc_line_type = DocLineType.TEXT_HEADER
                self.content = self.content[1:].strip()
            elif self.content == "":
                self.doc_line_type = DocLineType.TEXT_EMPTY
            elif self.content.startswith("```"):
                if in_doctest:
                    self.doc_line_type = DocLineType.CODE_END
                else:
                    self.doc_line_type = DocLineType.CODE_BEGIN
            else:
                if in_doctest:
                    self.doc_line_type = DocLineType.CODE
        elif sline.startswith("//"):
            # A comment but not a doc comment
            self.comment = True
            self.content = sline[2:].strip()
        else:
            # A normal line of code, so check if it defines an item
            if "fn " in sline:
                self.item_type = ItemType.FUNCTION
            elif "struct " in sline:
                self.item_type = ItemType.STRUCT
            elif "enum " in sline:
                self.item_type = ItemType.ENUM
            elif "trait " in sline:
                self.item_type = ItemType.TRAIT
            elif "type " in sline:
                self.item_type = ItemType.TYPE
            elif "impl " in sline or "impl<" in sline:
                self.item_type = ItemType.IMPL
            elif "use " in sline:
                self.item_type = ItemType.USE

    def is_doc_comment(self) -> bool:
        """
        Returns whether this line is any kind of doc comment.
        """
        return self.doc_line_type is not DocLineType.NONE

    def format(self) -> str:
        """
        Returns the line for printing.
        """
        if self.is_doc_comment():
            pref = "/// "
            if self.doc_line_type is DocLineType.TEXT_HEADER:
                pref += "# "
            return pref + self.content
        elif self.comment:
            return "// " + self.content
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
                if rust_line.doc_line_type == DocLineType.CODE_BEGIN:
                    in_doctest = True
                elif rust_line.doc_line_type == DocLineType.CODE_END:
                    in_doctest = False

                lines.append(rust_line)

        self.file_path = file_path
        self.lines = lines

    def format_line(self, line_num: int):
        """
        Print the line with the line number.
        """
        return self.file_path + ":" + str(line_num + 1) + " " + self.lines[line_num].format()

    def doc_comment_lines(self) -> Tuple[int, RustLine]:
        """
        Generator over the first lines of every doc comment.
        """
        for ln, line in enumerate(self.lines):
            # First check for multiple sentences in the doc introduction
            if line.is_doc_comment() and (ln == 0 or not self.lines[ln-1].is_doc_comment()):
                yield (ln, line)


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


class IsolatedIntroLint(Lint):
    """
    The introduction to every doc comment must be a single isolated sentence.
    """

    def __init__(self):
        self.name = "isolated_intro_sentence"
        self.description = self.__class__.__doc__.strip()

    def check_file(self, source_file: SourceFile):
        lines = source_file.lines

        for ln, line in source_file.doc_comment_lines():
            # Append the following lines until a non-normal doc comment line is found.
            long_line = line.content
            for next_line in lines[ln + 1:]:
                if next_line.doc_line_type is not DocLineType.TEXT:
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


class CompleteSentencesLint(Lint):
    """
    Text must be complete sentences.
    """

    def __init__(self):
        self.name = "complete_sentences"
        self.description = self.__class__.__doc__.strip()

    def check_file(self, source_file: SourceFile):
        # We group all normal text between non-normal delimiters (e.g. empty lines
        # or headers) and look for at least complete sentences for these section.
        # Note that we cannot always tell if a period is missing due to needing
        # to allow for proper case words in the middle of sentences.
        for ln, line in source_file.doc_comment_lines():
            sections = []
            text_lines = []
            for lline in source_file.lines[ln:]:
                if lline.doc_line_type is DocLineType.TEXT:
                    # Normal text, so just add it
                    text_lines.append(lline.content)
                else:
                    if len(text_lines) > 0:
                        sections.append(text_lines)
                    if lline.doc_line_type is DocLineType.NONE:
                        # End of this doc comment altogether
                        break
                    else:
                        # Some other doc comment line (e.g. header, empty)
                        text_lines = []

            # Now go through sections and make sentences to analyze
            for lines in sections:
                # Split into individual sentences and check them
                sentences = " ".join(lines).split(". ")
                for sentence in sentences:
                    words = sentence.split()

                    # Ensure that the first word is capitalized
                    if words[0].isalpha() and not words[0].isupper() and not words[0].istitle():
                        #print("FIRST WORD", words[0])
                        self.alert(source_file, ln)
                        break

                # Lastly, ensure that the final sentence does end in period
                if sentences[-1][-1] != ".":
                    #print("LAST PERIOD", sentences[-1])
                    self.alert(source_file, ln)


class CrossRefLint(Lint):
    """
    All cross references must use the Markdown code font.
    """

    def __init__(self):
        self.name = "cross_ref_code"
        self.description = self.__class__.__doc__.strip()

    def check_file(self, source_file: SourceFile):
        lines = source_file.lines

        for ln, line in enumerate(lines):
            if line.doc_line_type in (DocLineType.TEXT, DocLineType.TEXT_HEADER):
                content = line.content
                while True:
                    idx = content.find("[")
                    if idx > -1:
                        # Found a cross reference
                        content = content[idx+1:]
                        split = content.split("]")
                        if len(split) == 0 or not split[1].startswith("("):
                            # Okay this is not an ordinary link.
                            if not split[0].startswith("`"):
                                self.alert(source_file, ln)
                    else:
                        break


class FunctionIntroVerbLint(Lint):
    """
    Function intro sentences must begin with an action verb describing what it does, e.g. 'Returns X' instead of 'Return X'.
    """

    def __init__(self):
        self.name = "function_intro_verb"
        self.description = self.__class__.__doc__.strip()

    def check_file(self, source_file: SourceFile):
        for ln, line in source_file.doc_comment_lines():
            # Now look for the first item line
            for item_line in source_file.lines[ln+1:]:
                item_type = item_line.item_type
                if item_line.is_doc_comment() or item_type is not ItemType.NONE:
                    if item_type is ItemType.FUNCTION:
                        # Now verify the that first word is proper case that ends in `s`
                        first_word = line.content.split()[0]
                        if not first_word.istitle() or first_word[-1] != "s":
                            self.alert(source_file, ln)
                    break


class SimpleReplaceLint(Lint):
    """
    Lint class for general terms that should not be used.

    Sub-class should define the following attributes in the constructor
    then call this constructor:
    name - Lint name (str)
    bad_term - The term to avoid (str)
    good_term - The term that should be used instead (str)
    """

    def __init__(self):
        # TODO: Need to finish this and and instance for `sub-functions` -> `internal functions`
        # TODO: There are currently instances of this in the code left intentionally to test.
        self.description = "Do not use the term`" + self.bad_term + \
            "`. Use `" + self.good_term + "` instead."

    def check_file(self, source_file: SourceFile):
        for ln, line in source_file.doc_comment_lines():
            # Now jut look for the term in all subsequent doc comment lines.
            for item_line in source_file.lines[ln+1:]:
                item_type = item_line.item_type
                if item_line.doc_comment or item_type is not ItemType.NONE:
                    if item_type is ItemType.FUNCTION:
                        # Now verify the that first word is proper case that ends in `s`
                        first_word = line.content.split()[0]
                        if not first_word.istitle() or first_word[-1] != "s":
                            self.alert(source_file, ln)
                    break


# All our lints.
lints = [IsolatedIntroLint(), CrossRefLint(), CompleteSentencesLint(),
         FunctionIntroVerbLint()]


def source_files() -> str:
    """
    Generator over all Rust source file paths.
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
    for source_path in ["src/aoc/iter.rs"]:
        # for source_path in source_files():
        source_file = SourceFile(source_path)

        for lint in lints:
            lint.check_file(source_file)
