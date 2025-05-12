import os
from collections import Counter
import string


def get_letter_stats(folder_path, num_letters=26, num_pairs=50):
    letter_counts = Counter()
    pair_counts = Counter()

    try:
        text_files = [f for f in os.listdir(folder_path) if f.endswith(".txt")]

        if not text_files:
            print("No text files found in the specified folder.")
            return

        for file_name in text_files:
            file_path = os.path.join(folder_path, file_name)
            try:
                with open(file_path, "r", encoding="utf-8") as file:
                    content = file.read().lower()
                    letters = [
                        char for char in content if char in string.ascii_lowercase
                    ]

                    letter_counts.update(letters)

                    for i in range(len(letters) - 1):
                        pair = letters[i] + letters[i + 1]
                        pair_counts.update([pair])
            except Exception as e:
                print(f"Error reading file {file_name}: {str(e)}")

        if not letter_counts:
            print("No letters found in the text files.")
            return

        print(f"\nMost common single letters in {folder_path}:")
        for letter, count in letter_counts.most_common(num_letters):
            print(f"'{letter}': {count} occurrences")

        total_letters = sum(letter_counts.values())
        print(f"Total letters counted: {total_letters}")

        if not pair_counts:
            print("No two-letter combinations found in the text files.")
            return

        print(f"\nMost common two-letter combinations in {folder_path}:")
        for pair, count in pair_counts.most_common(num_pairs):
            print(f"'{pair}': {count} occurrences")

        total_pairs = sum(pair_counts.values())
        print(f"Total two-letter combinations counted: {total_pairs}")

    except Exception as e:
        print(f"Error accessing folder {folder_path}: {str(e)}")


def main():
    # folder_path = input("Enter the folder path containing text files: ")
    folder_path = "corpus"

    if not os.path.isdir(folder_path):
        print("Invalid folder path. Please provide a valid directory.")
        return

    get_letter_stats(folder_path)


if __name__ == "__main__":
    main()
