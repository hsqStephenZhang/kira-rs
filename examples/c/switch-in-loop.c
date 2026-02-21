
int main() {
    int i = 0;
    int c = 0;
    while (i < 10) {
        i = i + 1;
        if (i == 1) {
            continue;
        }
        c = c + 1;
    }
    return c;
}
