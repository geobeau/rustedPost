import sqlite3
import os.path
import json


conDst = sqlite3.connect('data/dataset_fiction.sqlite')
curDst = conDst.cursor()

# if not os.path.isfile('data/book_dataset.sqlite'):
#     conSrc = sqlite3.connect('~/Downloads/mysqlite3.db') # Libgen database dump 
#     curSrc = conSrc.cursor()

#     # Create table
#     curDst.execute('''CREATE TABLE IF NOT EXISTS book
#                 (AuthorFamilyName text, AuthorName text, AuthorSurname text, Language text, Year text, Extension text, Title text, Publisher text, Edition text)''')

#     curSrc.execute('SELECT AuthorFamily1, AuthorName1, AuthorSurname1, Language, Year, Extension, Title, Publisher, Edition FROM main;')
#     total = 0
#     lines = curSrc.fetchmany(size=10000)
#     while lines:
#         total += len(lines)
#         curDst.executemany("INSERT INTO book VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)", lines)
#         conDst.commit()
#         lines = curSrc.fetchmany(size=10000)
#         print(f"Current: {total} lines")

#     print(f"Finished::: Ingested {total} lines")

COL = ["author_family_name", "author_first_name", "language", "year", "extension", "title", "publisher", "edition"]

# with open('data/dataset_json.txt', 'w') as dataset:
#     print("Generating the dataset file from sqlite")
#     curDst.execute('SELECT * FROM book;')
#     total = 0
#     line = curDst.fetchone()
#     while line:
#         pairs = []
#         for i in range(len(COL)):
#             pairs.append({"key":COL[i],"val": line[i]})
#         dataset.write(json.dumps({"label_pairs":pairs}) + "\n")
#         line = curDst.fetchone()
#         total += 1
#         if total % 10000 == 0:
#             print(f"Current: {total} lines")
#     print(f"Finished::: Generated {total} lines")


with open('data/dataset_json.txt', 'w') as json_dataset:
    with open('data/dataset_custom.txt', 'w', encoding="utf8") as custom_dataset:
        print("Generating the dataset file from sqlite")
        curDst.execute('SELECT Author, Author, Language, Year, Extension, Title, Publisher, Edition FROM fiction;')
        total = 0
        orginal = 0
        line = curDst.fetchone()
        while line:
            for author in line[0].split(';'):
                custom_pairs = []
                json_pairs = []
                for i in range(len(COL)):
                    if i == 0:
                        val = author.split(',')[0]
                    elif i == 1:
                        try:
                            val = author.split(',')[1]
                        except IndexError:
                            val = ''
                    else:
                        val = line[i]
                    val = val.replace('\n', '').replace('\\','\\\\').replace('"','\\"').strip()
                    custom_pairs.append(f"{COL[i]}=\"{val}\"")
                    json_pairs.append({"key":COL[i],"val": val})
                custom_record = "{" + f"{','.join(custom_pairs)}" + "}\n"
                custom_dataset.write(custom_record)
                json_dataset.write(json.dumps({"label_pairs":json_pairs}) + "\n")
                total += 1

            orginal += 1
            if orginal % 10000 == 0:
                print(f"Current: {total} lines (without duplicate: {orginal})")
        
            line = curDst.fetchone()

        print(f"Finished::: Generated {total} lines")