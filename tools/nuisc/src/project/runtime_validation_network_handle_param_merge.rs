use super::super::NetworkOwnedHandleRequirement;

pub(super) fn merge_network_owned_handle_requirement(
    lhs: NetworkOwnedHandleRequirement,
    rhs: NetworkOwnedHandleRequirement,
) -> Option<NetworkOwnedHandleRequirement> {
    use NetworkOwnedHandleRequirement as Req;
    match (lhs, rhs) {
        (Req::OwnedAny, other) | (other, Req::OwnedAny) => Some(other),
        (Req::Transport, Req::StreamTransport) | (Req::StreamTransport, Req::Transport) => {
            Some(Req::StreamTransport)
        }
        (lhs, rhs) if lhs == rhs => Some(lhs),
        _ => None,
    }
}
