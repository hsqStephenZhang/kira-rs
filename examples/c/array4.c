int main() {
    int a[10];
    int i = 0;
    while (i < 10) {
        a[i] = i;
        i = i + 1;
    }
    return a[5] == 5;
}
