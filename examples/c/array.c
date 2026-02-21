int main() {
    int a[5];
    int len = 5;
    int i = 0;
    while (i < len) {
        a[i] = i;
        i = i + 1;
    }

    int result = 0;
    int j = 0;
    while (j < len) {
        result = result + a[i];
        j = j + 1;
    }
    return result == 10;
}
