
int main() {
    int i = 0;
    int result = 0;
    {
        result = i;
        i = i + 1;
    }
    while (i < 4) {
        result = i;
        i = i + 1;
    }
    return result;
}
