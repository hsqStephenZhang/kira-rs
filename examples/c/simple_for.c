
int main() {
    int i = 0;
    int sum = 0;
    while (i < 11) {
        sum = sum + i;
        i = i + 1;
    }
    return sum == 55;
}
