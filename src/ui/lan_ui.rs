use super::*;

pub const MATCHMAKER_SERVICE_NAME_ONEPLAYER: &str = "sb1player";
pub const MATCHMAKER_SERVICE_NAME_TWOPLAYER: &str = "sb2player";

#[derive(HasSchema, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ServiceType {
    OnePlayer(u32),
    TwoPlayer(u32, u32),
}
impl Default for ServiceType {
    fn default() -> Self {
        Self::OnePlayer(0)
    }
}
impl ServiceType {
    pub fn service_name(&self) -> &str {
        match self {
            ServiceType::OnePlayer(_) => MATCHMAKER_SERVICE_NAME_ONEPLAYER,
            ServiceType::TwoPlayer(_, _) => MATCHMAKER_SERVICE_NAME_TWOPLAYER,
        }
    }
}

#[derive(HasSchema, Default, Clone, Copy, PartialEq, Eq)]
pub enum LanUIState {
    #[default]
    Host,
    Disconnected,
    Server(usize),
}
impl LanUIState {
    pub fn cycle_up(&mut self) {
        match self {
            Self::Disconnected => {}
            Self::Host => {}
            Self::Server(i) => {
                if let Some(reduced) = i.checked_sub(1) {
                    *i = reduced;
                } else {
                    *self = Self::Host;
                }
            }
        }
    }
    pub fn cycle_down(&mut self) {
        match self {
            Self::Disconnected => {}
            Self::Host => *self = Self::Server(0),
            Self::Server(i) => *i = i.saturating_add(1), // This is capped in the `show` function
        }
    }
}

#[derive(HasSchema, Clone, Copy, Default, PartialEq, Eq)]
pub struct LanUI {
    pub visible: bool,
    pub service: ServiceType,
    pub state: LanUIState,
    pub output: Option<LanUIState>,
}

impl SessionPlugin for LanUI {
    fn install(self, session: &mut SessionBuilder) {
        session.insert_resource(self);
    }
}
pub fn show(world: &World) {
    let LanUI {
        visible,
        state,
        output,
        service,
    } = &mut *world.resource_mut::<LanUI>();

    *output = None;

    if !*visible {
        return;
    }

    let textures = world.resource::<EguiTextures>();
    let ctx = world.resource::<EguiCtx>();
    let asset_server = world.resource::<AssetServer>();
    let root = asset_server.root::<Data>();
    let locale = &asset_server.get(root.localization);
    let mut matchmaker = world.resource_mut::<Matchmaker>();

    let inner_font = asset_server
        .get(root.font.primary_inner)
        .family_name
        .clone();
    let outer_font = asset_server
        .get(root.font.primary_outer)
        .family_name
        .clone();
    let inner = TextPainter::standard()
        .size(7.0)
        .family(inner_font.clone())
        .color(Color32::WHITE);
    let outer = TextPainter::standard().size(7.0).family(outer_font.clone());

    let splash_bg = root.menu.splash.bg;

    use egui::*;

    Area::new("lan_ui")
        .order(Order::Background)
        .interactable(false)
        .anchor(Align2::CENTER_CENTER, [0., 0.])
        .show(&ctx, |ui| {
            ui.image(load::SizedTexture::new(
                textures.get(splash_bg),
                root.screen_size.to_array(),
            ));
        });

    if let LanUIState::Disconnected = state {
        Area::new("disconnected-popup")
            .anchor(Align2::CENTER_CENTER, [0., 0.])
            .order(Order::Foreground)
            .show(&world.resource::<EguiCtx>(), |ui| {
                BorderedFrame::new(&root.menu.bframe)
                    .padding(Margin::same(50.0))
                    .show(ui, |ui| {
                        let text = "A Player Disconnected...";
                        let response =
                            ui.label(RichText::new(text).color(Color32::WHITE).font(FontId {
                                size: 7.0,
                                family: FontFamily::Name(inner_font.clone()),
                            }));
                        TextPainter::new(text)
                            .size(7.0)
                            .pos(response.rect.min)
                            .family(outer_font.clone())
                            .color(Color32::BLACK)
                            .paint(ui.painter())
                    })
            });
        return;
    }

    Area::new("lan_service")
        .anchor(Align2::CENTER_CENTER, [0., -70.0])
        .show(&ctx, |ui| {
            BorderedFrame::new(&root.menu.bframe)
                .padding(Margin::same(5.0))
                .show(ui, |ui| {
                    let text = match &service {
                        ServiceType::OnePlayer(..) => "1 Vs 1 Twin-Stick Controls",
                        ServiceType::TwoPlayer(..) => "2 Vs 2 Cooperative",
                    };
                    let response = ui.label(
                        RichText::new(text)
                            .font(FontId {
                                size: 7.0,
                                family: FontFamily::Name(inner_font.clone()),
                            })
                            .color(Color32::WHITE),
                    );
                    TextPainter::new(text)
                        .size(7.0)
                        .pos(response.rect.min)
                        .family(outer_font.clone())
                        .color(Color32::BLACK)
                        .paint(ui.painter())
                });
        });
    Area::new("lan_servers")
        .anchor(Align2::CENTER_CENTER, [0., 0.])
        .show(&ctx, |ui| {
            BorderedFrame::new(&root.menu.bframe)
                .padding(Margin::same(5.0))
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            let response = BorderedFrame::new(&root.menu.bframe).show(ui, |ui| {
                                let painter = ui.painter();

                                let text = if matchmaker.is_hosting() {
                                    "Cancel"
                                } else {
                                    "Host"
                                };
                                let rect = outer
                                    .clone()
                                    .text("Cancel")
                                    .pos(ui.cursor().min)
                                    .color(Color32::TRANSPARENT)
                                    .paint(painter);

                                outer
                                    .clone()
                                    .text(text)
                                    .pos(rect.center())
                                    .align2(Align2::CENTER_CENTER)
                                    .paint(painter);

                                inner
                                    .clone()
                                    .text(text)
                                    .pos(rect.center())
                                    .align2(Align2::CENTER_CENTER)
                                    .color(if *state == LanUIState::Host {
                                        Color32::YELLOW
                                    } else {
                                        Color32::WHITE
                                    })
                                    .paint(painter);

                                ui.allocate_rect(rect.expand(6.0), Sense::click());
                            });
                            if ctx.clicked_rect(response.response.rect) {
                                *output = Some(LanUIState::Host);
                            }
                            // This wasn't working for some reason.
                            if response.response.clicked() {
                                *output = Some(LanUIState::Host);
                            }
                            if response.response.hovered() {
                                *state = LanUIState::Host;
                            }

                            BorderedFrame::new(&root.menu.bframe)
                                .padding(Margin::same(4.0))
                                .margin(Margin::ZERO)
                                .show(ui, |ui| {
                                    ui.vertical(|ui| {
                                        inner
                                            .clone()
                                            .text("Host Name")
                                            .pos(ui.cursor().min)
                                            .paint(ui.painter());
                                        let rect = outer
                                            .clone()
                                            .text("Host Name")
                                            .pos(ui.cursor().min)
                                            .paint(ui.painter());

                                        ui.allocate_rect(rect, Sense::focusable_noninteractive());

                                        BorderedFrame::new(&root.menu.bframe)
                                            .padding(Margin::same(2.0))
                                            .margin(Margin::ZERO)
                                            .show(ui, |ui| {
                                                let outer_font_id = FontId {
                                                    size: 7.0,
                                                    family: FontFamily::Name(outer_font),
                                                };
                                                let inner_font_id = FontId {
                                                    size: 7.0,
                                                    family: FontFamily::Name(inner_font),
                                                };

                                                // TODO: do this at startup instead
                                                ui.style_mut().visuals.selection.bg_fill =
                                                    Color32::YELLOW;
                                                ui.style_mut().visuals.text_cursor.color =
                                                    Color32::YELLOW;

                                                let egui::widgets::text_edit::TextEditOutput {
                                                    response,
                                                    text_draw_pos,
                                                    ..
                                                } = TextEdit::singleline(&mut matchmaker.host_name)
                                                    .char_limit(24)
                                                    .font(outer_font_id.clone())
                                                    .text_color(Color32::TRANSPARENT)
                                                    .frame(false)
                                                    .show(ui);

                                                if response.changed() {
                                                    let mut new = String::new();
                                                    for char in
                                                        matchmaker.host_name.as_str().chars()
                                                    {
                                                        if allowed_character(&char) {
                                                            new.push(char);
                                                        } else if char == ' ' {
                                                            new.push('_')
                                                        } else {
                                                            new.push('?');
                                                        }
                                                    }
                                                    matchmaker.host_name = new;
                                                }

                                                ui.painter().text(
                                                    text_draw_pos,
                                                    Align2::LEFT_TOP,
                                                    matchmaker.host_name.clone(),
                                                    inner_font_id,
                                                    Color32::LIGHT_GRAY,
                                                );
                                                ui.painter().text(
                                                    text_draw_pos,
                                                    Align2::LEFT_TOP,
                                                    matchmaker.host_name.clone(),
                                                    outer_font_id,
                                                    Color32::BLACK,
                                                );
                                            });
                                    });
                                });
                        });
                        if matchmaker.is_hosting() {
                            let text = "Waiting for players...";

                            let rect = outer
                                .clone()
                                .text(text)
                                .pos(ui.cursor().min)
                                .color(if matchmaker.is_hosting() {
                                    Color32::BLACK
                                } else {
                                    Color32::TRANSPARENT
                                })
                                .paint(ui.painter());
                            inner
                                .clone()
                                .text(text)
                                .pos(ui.cursor().min)
                                .color(if matchmaker.is_hosting() {
                                    Color32::WHITE
                                } else {
                                    Color32::TRANSPARENT
                                })
                                .paint(ui.painter());
                            ui.advance_cursor_after_rect(rect);
                            return;
                        }

                        let servers = matchmaker.lan_servers();
                        if servers.is_empty() {
                            let text = "No servers found";
                            let rect = outer
                                .clone()
                                .text(text)
                                .pos(ui.cursor().min)
                                .color(Color32::BLACK)
                                .paint(ui.painter());
                            inner
                                .clone()
                                .text(text)
                                .pos(ui.cursor().min)
                                .paint(ui.painter());
                            ui.advance_cursor_after_rect(rect);
                        }

                        if let LanUIState::Server(i) = state {
                            if let Some(index) = matchmaker.lan_servers().len().checked_sub(1) {
                                *i = (*i).min(index);
                            } else {
                                *state = LanUIState::Host;
                            }
                        }

                        for (i, server) in matchmaker.lan_servers().iter().enumerate() {
                            let irsp = ui.horizontal(|ui| {
                                let color = if LanUIState::Server(i) == *state {
                                    Color32::YELLOW
                                } else {
                                    Color32::WHITE
                                };
                                let ping = if let Some(ping) = server.ping {
                                    format!("PING: {ping}")
                                } else {
                                    "PING: ?".to_string()
                                };
                                outer
                                    .clone()
                                    .text(ping.clone())
                                    .pos(ui.cursor().min)
                                    .paint(ui.painter());
                                let rect = inner
                                    .clone()
                                    .text(ping)
                                    .color(color)
                                    .pos(ui.cursor().min)
                                    .paint(ui.painter());
                                ui.advance_cursor_after_rect(rect);
                                outer
                                    .clone()
                                    .text(server.service.get_hostname())
                                    .pos(ui.cursor().min)
                                    .paint(ui.painter());
                                let rect = inner
                                    .clone()
                                    .color(color)
                                    .text(server.service.get_hostname())
                                    .pos(ui.cursor().min)
                                    .paint(ui.painter());
                                ui.advance_cursor_after_rect(rect);
                            });
                            let rect = irsp.response.rect;
                            if ctx.clicked_rect(rect) {
                                *output = Some(LanUIState::Server(i));
                            }
                            if ctx.hovered_rect(rect) {
                                *state = LanUIState::Server(i);
                            }
                        }
                    });
                });
        });
}

pub fn allowed_character(char: &char) -> bool {
    matches!(
        char,
        'a' | 'b'
            | 'c'
            | 'd'
            | 'e'
            | 'f'
            | 'g'
            | 'h'
            | 'i'
            | 'j'
            | 'k'
            | 'l'
            | 'm'
            | 'n'
            | 'o'
            | 'p'
            | 'q'
            | 'r'
            | 's'
            | 't'
            | 'u'
            | 'v'
            | 'w'
            | 'A'
            | 'B'
            | 'C'
            | 'D'
            | 'E'
            | 'F'
            | 'G'
            | 'H'
            | 'I'
            | 'J'
            | 'K'
            | 'L'
            | 'M'
            | 'N'
            | 'O'
            | 'P'
            | 'Q'
            | 'R'
            | 'S'
            | 'T'
            | 'U'
            | 'V'
            | 'W'
            | 'X'
            | 'Y'
            | 'Z'
            | '1'
            | '2'
            | '3'
            | '4'
            | '5'
            | '6'
            | '7'
            | '8'
            | '9'
            | '0'
            | '?'
            | '_'
    )
}
