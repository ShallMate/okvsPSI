import pandas as pd
import numpy as np

# Set the number of rows for each CSV
rows_csv1 = 500
rows_csv2 = 400

# Generate unique elements for each CSV
unique_elements_csv1 = set(np.random.randint(10000, 99999, rows_csv1 - 100))
unique_elements_csv2 = set(np.random.randint(10000, 99999, rows_csv2 - 100))

# Ensure no overlap between the unique elements of the two CSVs
while not unique_elements_csv1.isdisjoint(unique_elements_csv2):
    unique_elements_csv1 = set(np.random.randint(10000, 99999, rows_csv1 - 100))
    unique_elements_csv2 = set(np.random.randint(10000, 99999, rows_csv2 - 100))

# Generate 100 common elements
common_elements = np.random.randint(10000, 99999, 100)

# Combine unique and common elements for each CSV
data_csv1 = list(unique_elements_csv1) + list(common_elements)
data_csv2 = list(unique_elements_csv2) + list(common_elements)

# Shuffle the data to mix common elements
np.random.shuffle(data_csv1)
np.random.shuffle(data_csv2)

# Convert to pandas DataFrame
df_csv1 = pd.DataFrame(data_csv1, columns=['Value'])
df_csv2 = pd.DataFrame(data_csv2, columns=['Value'])

# Save to CSV
df_csv1.to_csv('./csv1.csv', index=False)
df_csv2.to_csv('./csv2.csv', index=False)

