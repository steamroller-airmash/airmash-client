use ws::Sender;

use error::*;
use gamestate::GameState;
use protocol::server::*;
use protocol::{ClientPacket, ServerPacket};
use protocol::{KeyCode, PlaneType, Protocol};

use std::borrow::Borrow;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub type ClientResult<P> = Result<(), ClientError<P>>;

pub struct ClientState<'a, P: Protocol> {
    pub(crate) state: &'a GameState,
    pub(crate) key_seq: Arc<AtomicUsize>,
    pub(crate) sender: Sender,
    pub(crate) protocol: P,
}

pub fn default_on_packet<'a, P, C>(
    client: &mut C,
    state: &ClientState<'a, P>,
    packet: &ServerPacket,
) -> ClientResult<P>
where
    P: Protocol,
    C: Client<P> + ?Sized,
{
    use self::ServerPacket::*;

    match packet {
        Login(x) => client.on_login(state, x),

        PlayerNew(x) => client.on_player_join(state, x),
        PlayerLeave(x) => client.on_player_leave(state, x),
        PlayerUpdate(x) => client.on_player_update(state, x),
        PlayerKill(x) => client.on_player_killed(state, x),
        PlayerRespawn(x) => client.on_player_respawn(state, x),

        EventStealth(x) => client.on_player_stealth(state, x),
        EventBoost(x) => client.on_player_boost(state, x),
        EventBounce(x) => client.on_player_bounce(state, x),

        MobDespawn(x) => client.on_mob_despawn(state, x),
        MobDespawnCoords(x) => client.on_mob_despawn_coords(state, x),

        EventLeaveHorizon(x) => client.on_entity_leavehorizon(state, x),

        ChatPublic(x) => client.on_player_chat(state, x),
        ChatTeam(x) => client.on_player_team_chat(state, x),
        ChatWhisper(x) => client.on_player_whisper(state, x),
        ChatSay(x) => client.on_player_say(state, x),

        _ => Ok(()),
    }
}

pub trait Client<P: Protocol> {
    fn on_packet<'a>(
        &mut self,
        state: &ClientState<'a, P>,
        packet: &ServerPacket,
    ) -> ClientResult<P> {
        default_on_packet(self, state, packet)
    }
    fn on_gameloop<'a>(&mut self, _state: &ClientState<'a, P>, _now: Instant) -> ClientResult<P> {
        Ok(())
    }

    fn on_open<'a>(&mut self, _state: &ClientState<'a, P>) -> ClientResult<P> {
        Ok(())
    }
    fn on_close<'a>(&mut self, _state: &ClientState<'a, P>) -> ClientResult<P> {
        Ok(())
    }

    fn on_login<'a>(&mut self, _state: &ClientState<'a, P>, _info: &Login) -> ClientResult<P> {
        Ok(())
    }
    fn on_player_join<'a>(
        &mut self,
        _state: &ClientState<'a, P>,
        _info: &PlayerNew,
    ) -> ClientResult<P> {
        Ok(())
    }
    fn on_player_leave<'a>(
        &mut self,
        _state: &ClientState<'a, P>,
        _info: &PlayerLeave,
    ) -> ClientResult<P> {
        Ok(())
    }
    fn on_player_update<'a>(
        &mut self,
        _state: &ClientState<'a, P>,
        _info: &PlayerUpdate,
    ) -> ClientResult<P> {
        Ok(())
    }
    fn on_player_killed<'a>(
        &mut self,
        _state: &ClientState<'a, P>,
        _info: &PlayerKill,
    ) -> ClientResult<P> {
        Ok(())
    }
    fn on_player_spectate<'a>(
        &mut self,
        _state: &ClientState<'a, P>,
        _info: &PlayerKill,
    ) -> ClientResult<P> {
        Ok(())
    }
    fn on_player_stealth<'a>(
        &mut self,
        _state: &ClientState<'a, P>,
        _info: &EventStealth,
    ) -> ClientResult<P> {
        Ok(())
    }
    fn on_player_boost<'a>(
        &mut self,
        _state: &ClientState<'a, P>,
        _info: &EventBoost,
    ) -> ClientResult<P> {
        Ok(())
    }
    fn on_player_bounce<'a>(
        &mut self,
        _state: &ClientState<'a, P>,
        _info: &EventBounce,
    ) -> ClientResult<P> {
        Ok(())
    }
    fn on_player_fire<'a>(
        &mut self,
        _state: &ClientState<'a, P>,
        _info: &PlayerFire,
    ) -> ClientResult<P> {
        Ok(())
    }
    fn on_player_hit<'a>(
        &mut self,
        _state: &ClientState<'a, P>,
        _info: &PlayerHit,
    ) -> ClientResult<P> {
        Ok(())
    }
    fn on_player_respawn<'a>(
        &mut self,
        _state: &ClientState<'a, P>,
        _info: &PlayerRespawn,
    ) -> ClientResult<P> {
        Ok(())
    }

    fn on_mob_despawn<'a>(
        &mut self,
        _state: &ClientState<'a, P>,
        _info: &MobDespawn,
    ) -> ClientResult<P> {
        Ok(())
    }
    fn on_mob_despawn_coords<'a>(
        &mut self,
        _state: &ClientState<'a, P>,
        _info: &MobDespawnCoords,
    ) -> ClientResult<P> {
        Ok(())
    }

    fn on_entity_leavehorizon<'a>(
        &mut self,
        _state: &ClientState<'a, P>,
        _info: &EventLeaveHorizon,
    ) -> ClientResult<P> {
        Ok(())
    }

    fn on_player_chat<'a>(
        &mut self,
        _state: &ClientState<'a, P>,
        _info: &ChatPublic,
    ) -> ClientResult<P> {
        Ok(())
    }
    fn on_player_say<'a>(
        &mut self,
        _state: &ClientState<'a, P>,
        _info: &ChatSay,
    ) -> ClientResult<P> {
        Ok(())
    }
    fn on_player_team_chat<'a>(
        &mut self,
        _state: &ClientState<'a, P>,
        _info: &ChatTeam,
    ) -> ClientResult<P> {
        Ok(())
    }
    fn on_player_whisper<'a>(
        &mut self,
        _state: &ClientState<'a, P>,
        _info: &ChatWhisper,
    ) -> ClientResult<P> {
        Ok(())
    }
}

impl<'a, P: Protocol> ClientState<'a, P> {
    pub fn state(&self) -> &'a GameState {
        self.state
    }
    pub fn protocol(&self) -> &P {
        &self.protocol
    }

    fn send_ws_frame(&self, frame: Vec<u8>) -> ClientResult<P> {
        self.sender.send(frame)?;
        Ok(())
    }

    pub fn send_packet_ref(&self, packet: &ClientPacket) -> ClientResult<P> {
        for frame in self
            .protocol
            .serialize_client(packet)
            .map_err(PacketSerializeError)?
        {
            self.send_ws_frame(frame)?;
        }

        Ok(())
    }

    pub fn send_packet<C: Into<ClientPacket>>(&self, packet: C) -> ClientResult<P> {
        self.send_packet_ref(&packet.into())
    }
}

impl<'a, P: Protocol> ClientState<'a, P> {
    pub fn chat<C: ToString>(&self, message: C) -> ClientResult<P> {
        use protocol::client::Chat;

        self.send_packet(Chat {
            text: message.to_string(),
        })
    }
    pub fn say<C: ToString>(&self, message: C) -> ClientResult<P> {
        use protocol::client::Say;

        self.send_packet(Say {
            text: message.to_string(),
        })
    }

    pub fn send_command<C, D>(&self, command: C, data: D) -> ClientResult<P>
    where
        C: ToString,
        D: ToString,
    {
        use protocol::client::Command;

        self.send_packet(Command {
            com: command.to_string(),
            data: data.to_string(),
        })
    }

    pub fn change_flag<F>(&self, flag: F) -> ClientResult<P>
    where
        F: ToString,
    {
        self.send_command("flag", flag)
    }

    pub fn enter_spectate(&self) -> ClientResult<P> {
        self.send_command("spectate", -1)
    }

    pub fn respawn(&self) -> ClientResult<P> {
        let id = self.state.me.id;
        let plane = self.state.players[&id].plane;

        self.switch_plane(plane)
    }

    pub fn switch_plane(&self, plane: PlaneType) -> ClientResult<P> {
        self.send_command("respawn", plane as u8)
    }

    pub fn set_key(&self, keycode: KeyCode, state: bool) -> ClientResult<P> {
        use protocol::client::Key;

        self.send_packet(Key {
            key: keycode,
            state: state,
            seq: self.key_seq.fetch_add(1, Ordering::Relaxed) as u32,
        })
    }

    pub fn press_key(&self, keycode: KeyCode) -> ClientResult<P> {
        self.set_key(keycode, true)
    }
    pub fn release_key(&self, keycode: KeyCode) -> ClientResult<P> {
        self.set_key(keycode, false)
    }
}

impl<'a, P: Protocol> ClientState<'a, P> {
    pub fn login_with_session_and_horizon(
        self,
        name: String,
        flag: String,
        session: Option<String>,
        horizon_x: u16,
        horizon_y: u16,
    ) -> ClientResult<P>
    where
        Self: Sized,
    {
        use protocol::client::Login;

        let packet = Login {
            name: name,
            flag: flag,
            session: session.unwrap_or("none".to_owned()),
            // Will get updated later
            protocol: self.protocol.version(),

            // These are usually ignored by the server
            horizon_x: horizon_x,
            horizon_y: horizon_y,
        };

        self.send_packet(packet)
    }

    pub fn login_with_session<N, X, F>(self, name: &N, flag: &F, session: X) -> ClientResult<P>
    where
        Self: Sized,
        N: ToOwned<Owned = String> + ?Sized,
        X: Into<Option<String>>,
        F: ToOwned<Owned = String> + ?Sized,
        String: Borrow<N> + Borrow<F>,
    {
        self.login_with_session_and_horizon(
            name.to_owned(),
            flag.to_owned(),
            session.into(),
            4500,
            4500,
        )
    }

    pub fn login<N, F>(self, name: &N, flag: &F) -> ClientResult<P>
    where
        Self: Sized,
        N: ToOwned<Owned = String> + ?Sized,
        F: ToOwned<Owned = String> + ?Sized,
        String: Borrow<N> + Borrow<F>,
    {
        self.login_with_session(name, flag, None)
    }
}
