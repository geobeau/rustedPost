import requests
import csv
import json


try:
    with open('data/dataset.csv', 'x') as csvfile:
        print("Downloading dataset from github")
        r = requests.get("https://github.com/uchidalab/book-dataset/raw/master/Task2/book32-listing.csv")
        # format is "[AMAZON INDEX (ASIN)}","[FILENAME]","[IMAGE URL]","[TITLE]","[AUTHOR]","[CATEGORY ID]","[CATEGORY]"

        r.raise_for_status()
        csvfile.write(r.text)
except FileExistsError:
    print("Dataset already downloaded (delete dataset.csv if you want to redownload source data)")

with open('data/dataset.txt', 'w') as dataset:
    with open('data/dataset.csv', newline="\n", encoding="ISO-8859-1") as csvfile:
        books = csv.reader(csvfile, dialect=csv.unix_dialect, delimiter=",", quotechar='"')
        print("Generating the dataset file")
        for book in books:
            dataset.write(json.dumps({"label_pair":[{"key":"title","val":book[3]},{"key":"author","val":book[4]},{"key":"category_id","val":book[5]},{"key":"category","val":book[6]}]}) + "\n")
        
