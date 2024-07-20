export function shorten(str: string, num: number): string {
    var res = str.substring(0, num-3);
    res += "...";
    return res;
}

export interface ContainerStats {
    container_id: string;
    container: string;
    cpu_perc: string;
    mem_usage_limit: string;
    mem_perc: string;
    net_io: string;
    block_io: string;
    pids: string;
    image: string;
    command: string;
    created_at: string;
    status: string;
    ports: string;
    names: string;
}
  