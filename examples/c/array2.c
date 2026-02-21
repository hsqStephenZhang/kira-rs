int main() {
    int a[4][5];
    int row = 4;
    int col = 5;
    int i = 0;
    while (i < row) {
        int j = 0;
        while (j < col) {
            a[i][j] = i * j;
            j = j + 1;
        }
        i = i + 1;
    }
    return a[2][3] == 6;
}
