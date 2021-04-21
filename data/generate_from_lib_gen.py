import sqlite3
import os.path
import json


conDst = sqlite3.connect('data/book_dataset.sqlite')
curDst = conDst.cursor()

if not os.path.isfile('data/book_dataset.sqlite'):
    conSrc = sqlite3.connect('~/Downloads/mysqlite3.db') # Libgen database dump 
    curSrc = conSrc.cursor()

    # Create table
    curDst.execute('''CREATE TABLE IF NOT EXISTS book
                (AuthorFamilyName text, AuthorName text, AuthorSurname text, Language text, Year text, Extension text, Title text, Publisher text, Edition text)''')

    curSrc.execute('SELECT AuthorFamily1, AuthorName1, AuthorSurname1, Language, Year, Extension, Title, Publisher, Edition FROM main;')
    total = 0
    lines = curSrc.fetchmany(size=10000)
    while lines:
        total += len(lines)
        curDst.executemany("INSERT INTO book VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)", lines)
        conDst.commit()
        lines = curSrc.fetchmany(size=10000)
        print(f"Current: {total} lines")

    print(f"Finished::: Ingested {total} lines")

COL = ["author_family_name", "author_first_name", "author_surname", "language", "year", "extension", "title", "publisher", "edition"]

with open('data/dataset.txt', 'w') as dataset:
    print("Generating the dataset file from sqlite")
    curDst.execute('SELECT * FROM book;')
    total = 0
    line = curDst.fetchone()
    while line:
        pairs = []
        for i in range(len(COL)):
            pairs.append({"key":COL[i],"val": line[i]})
        dataset.write(json.dumps({"label_pair":pairs}) + "\n")
        line = curDst.fetchone()
        total += 1
        if total % 10000 == 0:
            print(f"Current: {total} lines")
    print(f"Finished::: Generated {total} lines")