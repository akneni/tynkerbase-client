import { NodeInfoCard } from "../molecules/molecules"


interface NodeMgmtPageProps {
    id: string,
    name: string,
    addr: string,
    is_active: boolean
}
export function NodeMgmtPage(props: NodeMgmtPageProps[]) {

    var nodes = [{
        id: "a698b8c70d08c9d8e098a9f8a",
        name: "Node 1",
        addr: "192.0.0.1",
        is_active: true,
    }];

    // props.push(nodes);

    
    return (<>
        <div>
            {nodes.map(d => (<NodeInfoCard id={d.id} name={d.name} active={d.is_active} ip_addr={d.addr} />))}
        </div>
    </>)
}