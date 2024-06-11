import csv


open_prices = []
high_prices = []
low_prices = []
close_prices = []

with open("test.csv", "r") as csvfile:
    reader = csv.reader(csvfile)
    next(reader)
    for row in reader:
        open_prices.append(float(row[1]))
        high_prices.append(float(row[2]))
        low_prices.append(float(row[3]))
        close_prices.append(float(row[4]))


# for price in open_prices:
#     print(price)
