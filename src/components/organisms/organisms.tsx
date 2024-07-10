import { NodeInfoCard } from "../molecules/molecules"

export function NodeMgmtPage() {

    let nodes = [
        {
            id: "a698b8c70d08c9d8e098a9f8a",
            name: "Node 1",
            active: true,
            ip_addr: "192.0.0.1",
        }
    ]

    return (<>
        <div>
            {nodes.map(d => (<NodeInfoCard id={d.id} name={d.name} active={d.active} ip_addr={d.ip_addr} />))}
        </div>
    </>)
}