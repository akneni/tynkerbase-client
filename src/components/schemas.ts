export interface Node {
    node_id: string,
    name: string,
    email: string,
    addr: string,
    status: string,
}

export interface NodeDiags {
    node_id: string,
    name: string,
    manufacturer?: string,
    cpu?: string,
    cpu_arc?: string,
    hardware_threads?: string,
    l1_cache_d?: string,
    l1_cache_i?: string,
    l2_cache?: string,
    l3_cache?: string,
    mem_total?: string,
    mem_free?: string,
}