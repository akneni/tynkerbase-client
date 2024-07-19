export function shorten(str: string, num: number): string {
    var res = str.substring(0, num-3);
    res += "...";
    return res;
}