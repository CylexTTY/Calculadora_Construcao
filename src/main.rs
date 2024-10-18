// main.rs

use eframe::egui;
use egui::{Color32, RichText};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

const LARGURA_TRELICA: Decimal = dec!(0.37);
const LARGURA_ISOPOR: Decimal = dec!(0.42);
const COMPRIMENTO_STEP: Decimal = dec!(0.2);
const PESO_AREIA_POR_M3: Decimal = dec!(1450);
const PESO_SACO_AREIA: Decimal = dec!(20);
const PESO_SACO_CIMENTO: Decimal = dec!(50);

#[derive(Serialize, Deserialize, Default)]
struct Configuracao {
    fator_argamassa_simples: String,
    fator_argamassa_dupla: String,
    coeficiente_rejunte: String,
}

struct CalculadoraConstrucao {
    calculadora_laje: CalculadoraLaje,
    calculadora_piso: CalculadoraPiso,
    calculadora_forro: CalculadoraForro,
    calculadora_materiais: CalculadoraMateriais,
    calculadora_basica: CalculadoraBasica,
    config: Configuracao,
    modo_atual: Modo,
}

#[derive(PartialEq)]
enum Modo {
    Laje,
    Piso,
    Forro,
    Materiais,
    Basica,
}

struct CalculadoraLaje {
    tipo_laje: TipoLaje,
    ambientes: Vec<AmbienteLaje>,
    resultado: String,
}

struct CalculadoraPiso {
    area_caixa: String,
    ambientes: Vec<Ambiente>,
    calcular_piso: bool,
    calcular_argamassa: bool,
    calcular_rejunte: bool,
    resultado: String,
    // Campos para o cálculo do rejunte
    tile_width: String,      // largura do revestimento (mm)
    tile_length: String,     // comprimento do revestimento (mm)
    tile_thickness: String,  // espessura do revestimento (mm)
    joint_spacing: String,   // espaçamento das juntas (mm)
    rejunte_coeficiente: String,
    // Campo para o cálculo da argamassa
    application_method: ApplicationMethod,
    argamassa_fator: String,
}

#[derive(PartialEq, Clone, Copy)]
enum ApplicationMethod {
    SingleSided,
    DoubleSided,
}

struct CalculadoraMateriais {
    modo_entrada: ModoEntradaMateriais,
    volume_concreto: String,
    quantidade_cimento: String,
    quantidade_areia_m3: String,
    quantidade_areia_sacos: String,
    quantidade_pedra_m3: String,
    quantidade_pedra_sacos: String,
    proporcao_cimento: String,
    proporcao_areia: String,
    proporcao_pedra: String,
    resultado: String,
}

struct CalculadoraBasica {
    display: String,
    memoria: Decimal,
    operacao_pendente: Option<char>,
    ultimo_numero: Decimal,
    limpar_na_proxima_entrada: bool,
    historico: VecDeque<String>,
}

struct CalculadoraForro {
    ambientes: Vec<AmbienteForro>,
    resultado: String,
    direcao_global: DirecaoForro,
}

#[derive(PartialEq, Clone, Copy)]
enum TipoLaje {
    Trelica,
    Isopor,
}

#[derive(PartialEq, Clone, Copy)]
enum ModoEntradaMateriais {
    VolumeConcreto,
    QuantidadeMateriais,
}

#[derive(Clone)]
struct Ambiente {
    largura: String,
    comprimento: String,
}

#[derive(Clone)]
struct AmbienteForro {
    largura: String,
    comprimento: String,
}

#[derive(Clone)]
struct AmbienteLaje {
    largura: String,
    comprimento: String,
    direcao: DirecaoLaje,
}

#[derive(PartialEq, Clone, Copy)]
enum DirecaoForro {
    MaiorLado,
    MenorLado,
}

#[derive(PartialEq, Clone, Copy)]
enum DirecaoLaje {
    MaiorLado,
    MenorLado,
}

impl Default for CalculadoraConstrucao {
    fn default() -> Self {
        let config: Configuracao = confy::load("calculadora_construcao", None).unwrap_or_default();

        Self {
            calculadora_laje: CalculadoraLaje::default(),
            calculadora_piso: CalculadoraPiso::with_config(&config),
            calculadora_materiais: CalculadoraMateriais::default(),
            calculadora_basica: CalculadoraBasica::new(),
            calculadora_forro: CalculadoraForro::default(),
            config,
            modo_atual: Modo::Laje,
        }
    }
}

impl Default for CalculadoraLaje {
    fn default() -> Self {
        Self {
            tipo_laje: TipoLaje::Trelica,
            ambientes: vec![AmbienteLaje::default()],
            resultado: String::new(),
        }
    }
}

impl Default for AmbienteLaje {
    fn default() -> Self {
        Self {
            largura: String::new(),
            comprimento: String::new(),
            direcao: DirecaoLaje::MenorLado,
        }
    }
}

impl CalculadoraPiso {
    fn with_config(config: &Configuracao) -> Self {
        let argamassa_fator = match config.fator_argamassa_simples.as_str() {
            "" => "5.0".to_string(),
            s => s.to_string(),
        };

        let rejunte_coeficiente = if !config.coeficiente_rejunte.is_empty() {
            config.coeficiente_rejunte.clone()
        } else {
            "1.58".to_string()
        };

        Self {
            area_caixa: String::new(),
            ambientes: vec![Ambiente::default()],
            calcular_piso: true,
            calcular_argamassa: false,
            calcular_rejunte: false,
            resultado: String::new(),
            tile_width: String::new(),
            tile_length: String::new(),
            tile_thickness: "3".to_string(),
            joint_spacing: "2".to_string(),
            rejunte_coeficiente,
            application_method: ApplicationMethod::SingleSided,
            argamassa_fator,
        }
    }
}

impl Default for CalculadoraMateriais {
    fn default() -> Self {
        Self {
            modo_entrada: ModoEntradaMateriais::VolumeConcreto,
            volume_concreto: String::new(),
            quantidade_cimento: String::new(),
            quantidade_areia_m3: String::new(),
            quantidade_areia_sacos: String::new(),
            quantidade_pedra_m3: String::new(),
            quantidade_pedra_sacos: String::new(),
            proporcao_cimento: "1".to_string(),
            proporcao_areia: "2".to_string(),
            proporcao_pedra: "2".to_string(),
            resultado: String::new(),
        }
    }
}

impl Default for Ambiente {
    fn default() -> Self {
        Self {
            largura: String::new(),
            comprimento: String::new(),
        }
    }
}

impl Default for CalculadoraForro {
    fn default() -> Self {
        Self {
            ambientes: vec![AmbienteForro::default()],
            resultado: String::new(),
            direcao_global: DirecaoForro::MaiorLado,
        }
    }
}

impl Default for AmbienteForro {
    fn default() -> Self {
        Self {
            largura: String::new(),
            comprimento: String::new(),
        }
    }
}

impl eframe::App for CalculadoraConstrucao {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Define o tema escuro
        ctx.set_visuals(egui::Visuals::dark());

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.heading(
                    RichText::new("Calculadora de Construção")
                        .color(Color32::from_rgb(80, 160, 255)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Resetar Valores Padrão").clicked() {
                        confy::store("calculadora_construcao", None, &Configuracao::default())
                            .unwrap();
                        *self = CalculadoraConstrucao::default();
                    }
                });
            });
            ui.add_space(10.0);
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Menu");
            ui.separator();

            ui.spacing_mut().item_spacing.y = 10.0;

            if ui
                .selectable_label(self.modo_atual == Modo::Laje, "Calculadora de Laje")
                .clicked()
            {
                self.modo_atual = Modo::Laje;
            }
            if ui
                .selectable_label(self.modo_atual == Modo::Piso, "Calculadora de Piso")
                .clicked()
            {
                self.modo_atual = Modo::Piso;
            }
            if ui
                .selectable_label(self.modo_atual == Modo::Forro, "Calculadora de Forro")
                .clicked()
            {
                self.modo_atual = Modo::Forro;
            }
            if ui
                .selectable_label(
                    self.modo_atual == Modo::Materiais,
                    "Calculadora de Materiais",
                )
                .clicked()
            {
                self.modo_atual = Modo::Materiais;
            }
            if ui
                .selectable_label(self.modo_atual == Modo::Basica, "Calculadora Básica")
                .clicked()
            {
                self.modo_atual = Modo::Basica;
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                match self.modo_atual {
                    Modo::Laje => self.mostrar_calculadora_laje(ui),
                    Modo::Piso => self.mostrar_calculadora_piso(ui),
                    Modo::Forro => self.mostrar_calculadora_forro(ui),
                    Modo::Materiais => self.mostrar_calculadora_materiais(ui),
                    Modo::Basica => self.mostrar_calculadora_basica(ui, ctx),
                }
            });
        });
    }
}

impl CalculadoraConstrucao {
    fn mostrar_calculadora_laje(&mut self, ui: &mut egui::Ui) {
        ui.heading(
            RichText::new("Calculadora de Laje")
                .color(Color32::from_rgb(80, 160, 255)),
        );

        ui.add_space(10.0);

        egui::Grid::new("tipo_laje_grid")
            .num_columns(2)
            .spacing([10.0, 10.0])
            .min_col_width(100.0)
            .show(ui, |ui| {
                ui.label("Tipo de Laje:");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.calculadora_laje.tipo_laje, TipoLaje::Trelica, "Treliça");
                    ui.radio_value(&mut self.calculadora_laje.tipo_laje, TipoLaje::Isopor, "Isopor");
                });
                ui.end_row();
            });

        ui.add_space(10.0);

        let mut ambiente_para_remover = None;
        let ambientes_len = self.calculadora_laje.ambientes.len();

        egui::ScrollArea::vertical()
            .id_source("laje_ambientes_scroll")
            .show(ui, |ui| {
                for index in 0..ambientes_len {
                    let ambiente = &mut self.calculadora_laje.ambientes[index];
                    ui.group(|ui| {
                        ui.heading(
                            RichText::new(format!("Ambiente {}:", index + 1))
                                .color(Color32::from_rgb(80, 160, 255)),
                        );
                        ui.add_space(5.0);

                        egui::Grid::new(format!("ambiente_grid_{}", index))
                            .num_columns(2)
                            .spacing([10.0, 10.0])
                            .min_col_width(ui.available_width() / 2.0 - 20.0)
                            .show(ui, |ui| {
                                ui.label("Largura (m):");
                                ui.add(
                                    egui::TextEdit::singleline(&mut ambiente.largura)
                                        .desired_width(ui.available_width() / 2.0 - 20.0),
                                );
                                ui.end_row();

                                ui.label("Comprimento (m):");
                                ui.add(
                                    egui::TextEdit::singleline(&mut ambiente.comprimento)
                                        .desired_width(ui.available_width() / 2.0 - 20.0),
                                );
                                ui.end_row();

                                ui.label("Direção de Instalação:");
                                ui.horizontal(|ui| {
                                    ui.radio_value(
                                        &mut ambiente.direcao,
                                        DirecaoLaje::MenorLado,
                                        "Menor Lado",
                                    );
                                    ui.radio_value(
                                        &mut ambiente.direcao,
                                        DirecaoLaje::MaiorLado,
                                        "Maior Lado",
                                    );
                                });
                                ui.end_row();
                            });

                        ui.add_space(5.0);

                        ui.horizontal(|ui| {
                            if ui.button("Remover Ambiente").clicked() && ambientes_len > 1 {
                                ambiente_para_remover = Some(index);
                            }
                        });
                    });

                    ui.add_space(10.0);
                }
            });

        if let Some(index) = ambiente_para_remover {
            self.calculadora_laje.ambientes.remove(index);
        }

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("Adicionar Ambiente").clicked() {
                self.calculadora_laje.ambientes.push(AmbienteLaje::default());
            }
            if ui.button("Calcular Laje").clicked() {
                self.calculadora_laje.calcular();
            }
            if ui.button("Resetar Valores").clicked() {
                self.calculadora_laje = CalculadoraLaje::default();
            }
        });

        ui.add_space(15.0);
        ui.separator();
        ui.add_space(15.0);

        egui::ScrollArea::vertical()
            .id_source("laje_resultado_scroll")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.label(
                    RichText::new(&self.calculadora_laje.resultado)
                        .color(Color32::WHITE),
                );
            });
    }

    fn mostrar_calculadora_piso(&mut self, ui: &mut egui::Ui) {
        ui.heading(
            RichText::new("Calculadora de Piso")
                .color(Color32::from_rgb(80, 160, 255)),
        );

        ui.add_space(10.0);

        // Opções de cálculo
        ui.horizontal(|ui| {
            if ui.checkbox(&mut self.calculadora_piso.calcular_piso, "Calcular Piso").clicked() {
                self.calculadora_piso.validar_selecao('p');
            }
            if ui
                .checkbox(&mut self.calculadora_piso.calcular_argamassa, "Calcular Argamassa")
                .clicked()
            {
                self.calculadora_piso.validar_selecao('a');
            }
            if ui
                .checkbox(&mut self.calculadora_piso.calcular_rejunte, "Calcular Rejunte")
                .clicked()
            {
                self.calculadora_piso.validar_selecao('r');
            }
        });

        ui.add_space(10.0);

        // Mostrar ambientes sempre
        let mut ambiente_para_remover = None;
        let ambientes_len = self.calculadora_piso.ambientes.len();

        egui::ScrollArea::vertical()
            .id_source("piso_ambientes_scroll")
            .show(ui, |ui| {
                for index in 0..ambientes_len {
                    let ambiente = &mut self.calculadora_piso.ambientes[index];
                    ui.group(|ui| {
                        ui.heading(
                            RichText::new(format!("Ambiente {}:", index + 1))
                                .color(Color32::from_rgb(80, 160, 255)),
                        );
                        ui.add_space(5.0);

                        egui::Grid::new(format!("piso_ambiente_grid_{}", index))
                            .num_columns(2)
                            .spacing([10.0, 10.0])
                            .min_col_width(ui.available_width() / 2.0 - 20.0)
                            .show(ui, |ui| {
                                ui.label("Largura (m):");
                                ui.add(
                                    egui::TextEdit::singleline(&mut ambiente.largura)
                                        .desired_width(ui.available_width() / 2.0 - 20.0),
                                );
                                ui.end_row();

                                ui.label("Comprimento (m):");
                                ui.add(
                                    egui::TextEdit::singleline(&mut ambiente.comprimento)
                                        .desired_width(ui.available_width() / 2.0 - 20.0),
                                );
                                ui.end_row();
                            });

                        ui.add_space(5.0);

                        ui.horizontal(|ui| {
                            if ui.button("Remover Ambiente").clicked() && ambientes_len > 1 {
                                ambiente_para_remover = Some(index);
                            }
                        });
                    });

                    ui.add_space(10.0);
                }
            });

        if let Some(index) = ambiente_para_remover {
            self.calculadora_piso.ambientes.remove(index);
        }

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("Adicionar Ambiente").clicked() {
                self.calculadora_piso.ambientes.push(Ambiente::default());
            }
        });

        ui.add_space(10.0);

        // **Alteração aplicada aqui**
        if self.calculadora_piso.calcular_piso {
            ui.horizontal(|ui| {
                ui.label("Área da caixa (m²):");
                ui.add(
                    egui::TextEdit::singleline(&mut self.calculadora_piso.area_caixa)
                        .desired_width(100.0),
                );
            });

            ui.add_space(10.0);
        }

        if self.calculadora_piso.calcular_argamassa {
            ui.add_space(10.0);
            ui.label("Método de aplicação da argamassa:");
            ui.horizontal(|ui| {
                if ui
                    .radio_value(
                        &mut self.calculadora_piso.application_method,
                        ApplicationMethod::SingleSided,
                        "Aplicação Simples",
                    )
                    .clicked()
                {
                    self.atualizar_fator_argamassa();
                }
                if ui
                    .radio_value(
                        &mut self.calculadora_piso.application_method,
                        ApplicationMethod::DoubleSided,
                        "Colagem Dupla",
                    )
                    .clicked()
                {
                    self.atualizar_fator_argamassa();
                }
            });

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label("Fator de Argamassa (kg/m²):");
                ui.add(
                    egui::TextEdit::singleline(&mut self.calculadora_piso.argamassa_fator)
                        .desired_width(100.0)
                        .hint_text("Ex: 5.0"),
                )
                    .on_hover_text("Ajuste o fator de consumo de argamassa.");
                if ui.button("Salvar como Valor Padrão").clicked() {
                    self.salvar_fator_argamassa_padrao();
                }
            });
        }

        if self.calculadora_piso.calcular_rejunte {
            ui.add_space(10.0);
            ui.label("Dados do Revestimento para o Rejunte:");

            egui::Grid::new("rejunte_grid")
                .num_columns(2)
                .spacing([10.0, 10.0])
                .min_col_width(ui.available_width() / 2.0 - 20.0)
                .show(ui, |ui| {
                    ui.label("Largura do Revestimento (mm):");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.calculadora_piso.tile_width)
                            .desired_width(ui.available_width() / 2.0 - 20.0),
                    );
                    ui.end_row();

                    ui.label("Comprimento do Revestimento (mm):");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.calculadora_piso.tile_length)
                            .desired_width(ui.available_width() / 2.0 - 20.0),
                    );
                    ui.end_row();

                    ui.label("Espessura do Revestimento (mm):");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.calculadora_piso.tile_thickness)
                            .desired_width(ui.available_width() / 2.0 - 20.0),
                    );
                    ui.end_row();

                    ui.label("Espaçamento das Juntas (mm):");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.calculadora_piso.joint_spacing)
                            .desired_width(ui.available_width() / 2.0 - 20.0),
                    );
                    ui.end_row();

                    ui.label("Coeficiente de Rejunte:");
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.calculadora_piso.rejunte_coeficiente)
                                .desired_width(100.0)
                                .hint_text("Ex: 1.58"),
                        )
                            .on_hover_text("Coeficiente de Rejunte é usado na fórmula:\nkg/m² = ((L + C) x E x J x Coeficiente) / (L x C)\nOnde:\n  L = Largura do revestimento (mm)\n  C = Comprimento do revestimento (mm)\n  E = Espessura do revestimento (mm)\n  J = Espaçamento das juntas (mm)\nAjuste o coeficiente conforme necessário.");
                        if ui.button("Salvar como Valor Padrão").clicked() {
                            self.salvar_coeficiente_rejunte_padrao();
                        }
                    });
                    ui.end_row();
                });
        }

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("Calcular").clicked() {
                self.calculadora_piso.calcular();
            }
            if ui.button("Resetar Valores").clicked() {
                self.calculadora_piso = CalculadoraPiso::with_config(&self.config);
            }
        });

        ui.add_space(15.0);
        ui.separator();
        ui.add_space(15.0);

        egui::ScrollArea::vertical()
            .id_source("piso_resultado_scroll")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.label(
                    RichText::new(&self.calculadora_piso.resultado)
                        .color(Color32::WHITE),
                );
            });
    }

    fn atualizar_fator_argamassa(&mut self) {
        self.calculadora_piso.argamassa_fator = match self.calculadora_piso.application_method {
            ApplicationMethod::SingleSided => {
                if !self.config.fator_argamassa_simples.is_empty() {
                    self.config.fator_argamassa_simples.clone()
                } else {
                    "5.0".to_string()
                }
            }
            ApplicationMethod::DoubleSided => {
                if !self.config.fator_argamassa_dupla.is_empty() {
                    self.config.fator_argamassa_dupla.clone()
                } else {
                    "7.0".to_string()
                }
            }
        };
    }

    fn salvar_fator_argamassa_padrao(&mut self) {
        match self.calculadora_piso.application_method {
            ApplicationMethod::SingleSided => {
                self.config.fator_argamassa_simples =
                    self.calculadora_piso.argamassa_fator.clone();
            }
            ApplicationMethod::DoubleSided => {
                self.config.fator_argamassa_dupla = self.calculadora_piso.argamassa_fator.clone();
            }
        }
        confy::store("calculadora_construcao", None, &self.config).unwrap();
    }

    fn salvar_coeficiente_rejunte_padrao(&mut self) {
        self.config.coeficiente_rejunte = self.calculadora_piso.rejunte_coeficiente.clone();
        confy::store("calculadora_construcao", None, &self.config).unwrap();
    }

    fn mostrar_calculadora_materiais(&mut self, ui: &mut egui::Ui) {
        ui.heading(
            RichText::new("Calculadora de Materiais")
                .color(Color32::from_rgb(80, 160, 255)),
        );

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.radio_value(
                &mut self.calculadora_materiais.modo_entrada,
                ModoEntradaMateriais::VolumeConcreto,
                "Por Volume de Concreto",
            );
            ui.radio_value(
                &mut self.calculadora_materiais.modo_entrada,
                ModoEntradaMateriais::QuantidadeMateriais,
                "Por Quantidade de Materiais",
            );
        });

        ui.add_space(10.0);

        egui::Grid::new("materiais_grid")
            .num_columns(2)
            .spacing([10.0, 10.0])
            .min_col_width(ui.available_width() / 2.0 - 20.0)
            .show(ui, |ui| {
                match self.calculadora_materiais.modo_entrada {
                    ModoEntradaMateriais::VolumeConcreto => {
                        ui.label("Volume de Concreto (m³):");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.calculadora_materiais.volume_concreto)
                                .desired_width(ui.available_width() / 2.0 - 20.0),
                        );
                        ui.end_row();
                    }
                    ModoEntradaMateriais::QuantidadeMateriais => {
                        ui.label("Quantidade de Cimento (sacos):");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.calculadora_materiais.quantidade_cimento)
                                .desired_width(ui.available_width() / 2.0 - 20.0),
                        );
                        ui.end_row();

                        ui.label("Quantidade de Areia:");
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(
                                    &mut self.calculadora_materiais.quantidade_areia_m3,
                                )
                                    .desired_width(60.0),
                            );
                            ui.label("m³ ou");
                            ui.add(
                                egui::TextEdit::singleline(
                                    &mut self.calculadora_materiais.quantidade_areia_sacos,
                                )
                                    .desired_width(60.0),
                            );
                            ui.label("sacos (20kg)");
                        });
                        ui.end_row();

                        ui.label("Quantidade de Pedra:");
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(
                                    &mut self.calculadora_materiais.quantidade_pedra_m3,
                                )
                                    .desired_width(60.0),
                            );
                            ui.label("m³ ou");
                            ui.add(
                                egui::TextEdit::singleline(
                                    &mut self.calculadora_materiais.quantidade_pedra_sacos,
                                )
                                    .desired_width(60.0),
                            );
                            ui.label("sacos (20kg)");
                        });
                        ui.end_row();
                    }
                }

                ui.label("Proporção (Cimento:Areia:Pedra):");
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.calculadora_materiais.proporcao_cimento)
                            .desired_width(30.0),
                    );
                    ui.label(":");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.calculadora_materiais.proporcao_areia)
                            .desired_width(30.0),
                    );
                    ui.label(":");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.calculadora_materiais.proporcao_pedra)
                            .desired_width(30.0),
                    );
                });
                ui.end_row();
            });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("Calcular Materiais").clicked() {
                self.calculadora_materiais.calcular();
            }
            if ui.button("Resetar Valores").clicked() {
                self.calculadora_materiais = CalculadoraMateriais::default();
            }
        });

        ui.add_space(15.0);
        ui.separator();
        ui.add_space(15.0);

        egui::ScrollArea::vertical()
            .id_source("materiais_resultado_scroll")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.label(
                    RichText::new(&self.calculadora_materiais.resultado)
                        .color(Color32::WHITE),
                );
            });
    }

    fn mostrar_calculadora_basica(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading(
            RichText::new("Calculadora Básica")
                .color(Color32::from_rgb(80, 160, 255)),
        );

        ui.add_space(10.0);

        let available_width = ui.available_width();
        let button_width = available_width / 4.0 - 10.0;
        let button_height = 50.0;

        ui.add(
            egui::TextEdit::singleline(&mut self.calculadora_basica.display)
                .font(egui::TextStyle::Heading)
                .desired_width(available_width),
        );

        ui.add_space(10.0);

        let botoes = [
            ['7', '8', '9', '/'],
            ['4', '5', '6', '*'],
            ['1', '2', '3', '-'],
            ['0', '.', '=', '+'],
            ['C', 'M', '\u{8}', ' '], // '\u{8}' is backspace
        ];

        for linha in &botoes {
            ui.horizontal(|ui| {
                for &botao in linha {
                    if botao == ' ' {
                        ui.add_sized([button_width, button_height], egui::Label::new(""));
                    } else {
                        if ui
                            .add_sized(
                                [button_width, button_height],
                                egui::Button::new(botao.to_string()),
                            )
                            .clicked()
                        {
                            self.calculadora_basica.processar_entrada(botao);
                        }
                    }
                }
            });
            ui.add_space(5.0);
        }

        // Capturar entrada do teclado
        if let Some(c) = ctx.input(|i| {
            i.events.iter().find_map(|event| match event {
                egui::Event::Key {
                    key,
                    pressed: true,
                    ..
                } => key_to_char(*key),
                egui::Event::Text(text) => text.chars().next(),
                _ => None,
            })
        }) {
            self.calculadora_basica.processar_entrada(c);
        }

        ui.add_space(10.0);

        ui.label("Histórico:");
        egui::ScrollArea::vertical()
            .id_source("calculadora_basica_historico")
            .max_height(150.0)
            .show(ui, |ui| {
                for calculo in self.calculadora_basica.historico.iter() {
                    ui.label(calculo);
                }
            });
    }

    fn mostrar_calculadora_forro(&mut self, ui: &mut egui::Ui) {
        ui.heading(
            RichText::new("Calculadora de Forro de PVC")
                .color(Color32::from_rgb(80, 160, 255)),
        );

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Direção de Instalação Global:");
            ui.radio_value(
                &mut self.calculadora_forro.direcao_global,
                DirecaoForro::MaiorLado,
                "Maior Lado",
            );
            ui.radio_value(
                &mut self.calculadora_forro.direcao_global,
                DirecaoForro::MenorLado,
                "Menor Lado",
            );
        });

        ui.add_space(10.0);

        let mut ambiente_para_remover = None;
        let ambientes_len = self.calculadora_forro.ambientes.len();

        egui::ScrollArea::vertical()
            .id_source("forro_ambientes_scroll")
            .show(ui, |ui| {
                for index in 0..ambientes_len {
                    let ambiente = &mut self.calculadora_forro.ambientes[index];
                    ui.group(|ui| {
                        ui.heading(
                            RichText::new(format!("Ambiente {}:", index + 1))
                                .color(Color32::from_rgb(80, 160, 255)),
                        );
                        ui.add_space(5.0);

                        egui::Grid::new(format!("forro_ambiente_grid_{}", index))
                            .num_columns(2)
                            .spacing([10.0, 10.0])
                            .min_col_width(ui.available_width() / 2.0 - 20.0)
                            .show(ui, |ui| {
                                ui.label("Largura (m):");
                                ui.add(
                                    egui::TextEdit::singleline(&mut ambiente.largura)
                                        .desired_width(ui.available_width() / 2.0 - 20.0),
                                );
                                ui.end_row();

                                ui.label("Comprimento (m):");
                                ui.add(
                                    egui::TextEdit::singleline(&mut ambiente.comprimento)
                                        .desired_width(ui.available_width() / 2.0 - 20.0),
                                );
                                ui.end_row();
                            });

                        ui.add_space(5.0);

                        ui.horizontal(|ui| {
                            if ui.button("Remover Ambiente").clicked() && ambientes_len > 1 {
                                ambiente_para_remover = Some(index);
                            }
                        });
                    });

                    ui.add_space(10.0);
                }
            });

        if let Some(index) = ambiente_para_remover {
            self.calculadora_forro.ambientes.remove(index);
        }

        ui.add_space(10.0);

        // Mover o botão "Adicionar Ambiente" para cima
        ui.horizontal(|ui| {
            if ui.button("Adicionar Ambiente").clicked() {
                self.calculadora_forro.ambientes.push(AmbienteForro::default());
            }
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("Calcular Forro").clicked() {
                self.calculadora_forro.calcular();
            }
            if ui.button("Resetar Valores").clicked() {
                self.calculadora_forro = CalculadoraForro::default();
            }
        });

        ui.add_space(15.0);
        ui.separator();
        ui.add_space(15.0);

        egui::ScrollArea::vertical()
            .id_source("forro_resultado_scroll")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.label(
                    RichText::new(&self.calculadora_forro.resultado)
                        .color(Color32::WHITE),
                );
            });
    }
}

impl CalculadoraPiso {
    fn validar_selecao(&mut self, opcao: char) {
        let total_selecionado = self.calcular_piso as u8
            + self.calcular_argamassa as u8
            + self.calcular_rejunte as u8;

        if total_selecionado > 2 {
            // Só permite piso e mais um, ou apenas um
            match opcao {
                'p' => {
                    // Se selecionou piso, desativa as outras
                    self.calcular_argamassa = false;
                    self.calcular_rejunte = false;
                }
                'a' => {
                    // Se selecionou argamassa, desativa rejunte
                    self.calcular_rejunte = false;
                    if !self.calcular_piso && !self.calcular_argamassa {
                        self.calcular_piso = true;
                    }
                }
                'r' => {
                    // Se selecionou rejunte, desativa argamassa
                    self.calcular_argamassa = false;
                    if !self.calcular_piso && !self.calcular_rejunte {
                        self.calcular_piso = true;
                    }
                }
                _ => {}
            }
        }

        // Não permite selecionar argamassa e rejunte juntos
        if self.calcular_argamassa && self.calcular_rejunte {
            match opcao {
                'a' => self.calcular_rejunte = false,
                'r' => self.calcular_argamassa = false,
                _ => {}
            }
        }
    }

    fn calcular(&mut self) {
        self.resultado.clear();

        let mut area_total = Decimal::ZERO;

        for ambiente in &self.ambientes {
            let largura = match parse_decimal(&ambiente.largura) {
                Ok(v) => v,
                Err(_) => {
                    self.resultado = "Largura de ambiente inválida".to_string();
                    return;
                }
            };

            let comprimento = match parse_decimal(&ambiente.comprimento) {
                Ok(v) => v,
                Err(_) => {
                    self.resultado = "Comprimento de ambiente inválido".to_string();
                    return;
                }
            };

            area_total += largura * comprimento;
        }

        if self.calcular_piso {
            let area_caixa = match parse_decimal(&self.area_caixa) {
                Ok(v) => v,
                Err(_) => {
                    self.resultado = "Área da caixa inválida".to_string();
                    return;
                }
            };

            let caixas_necessarias = (area_total / area_caixa).ceil();
            let metragem_total = caixas_necessarias * area_caixa;
            let sobra_estimada = metragem_total - area_total;

            self.resultado.push_str(&format!(
                "Cálculo de Piso:\nÁrea da Caixa: {:.2} m²\nÁrea Total a Cobrir: {:.2} m²\n\
                Caixas Necessárias: {} ({:.2} m²)\nSobra Estimada: {:.2} m²\n\n",
                area_caixa,
                area_total,
                caixas_necessarias,
                metragem_total,
                sobra_estimada
            ));
        }

        if self.calcular_argamassa {
            let fator = match parse_decimal(&self.argamassa_fator) {
                Ok(v) => v,
                Err(_) => {
                    self.resultado = "Fator de argamassa inválido".to_string();
                    return;
                }
            };

            let argamassa_kg = area_total * fator;
            let sacos_necessarios = (argamassa_kg / dec!(20)).ceil();

            self.resultado.push_str(&format!(
                "Cálculo de Argamassa:\nÁrea Total: {:.2} m²\nMétodo de Aplicação: {}\n\
                Fator de Consumo: {:.2} kg/m²\nQuantidade de Argamassa Necessária: {:.2} kg\nSacos de 20kg Necessários: {}\n\n",
                area_total,
                if self.application_method == ApplicationMethod::SingleSided {
                    "Aplicação Simples"
                } else {
                    "Colagem Dupla"
                },
                fator,
                argamassa_kg,
                sacos_necessarios,
            ));
        }

        if self.calcular_rejunte {
            let tile_width = match parse_decimal(&self.tile_width) {
                Ok(v) => v,
                Err(_) => {
                    self.resultado = "Largura do revestimento inválida".to_string();
                    return;
                }
            };

            let tile_length = match parse_decimal(&self.tile_length) {
                Ok(v) => v,
                Err(_) => {
                    self.resultado = "Comprimento do revestimento inválida".to_string();
                    return;
                }
            };

            let tile_thickness = match parse_decimal(&self.tile_thickness) {
                Ok(v) => v,
                Err(_) => {
                    self.resultado = "Espessura do revestimento inválida".to_string();
                    return;
                }
            };

            let joint_spacing = match parse_decimal(&self.joint_spacing) {
                Ok(v) => v,
                Err(_) => {
                    self.resultado = "Espaçamento das juntas inválido".to_string();
                    return;
                }
            };

            let rejunte_coeficiente = match parse_decimal(&self.rejunte_coeficiente) {
                Ok(v) => v,
                Err(_) => {
                    self.resultado = "Coeficiente de rejunte inválido".to_string();
                    return;
                }
            };

            // Fórmula: kg/m² = (L+C) x E x J x Coef / (L x C)
            let numerator =
                (tile_width + tile_length) * tile_thickness * joint_spacing * rejunte_coeficiente;
            let denominator = tile_width * tile_length;
            let rejunte_por_m2 = numerator / denominator;

            let rejunte_total = rejunte_por_m2 * area_total * dec!(1.05); // Acrescentar 5% de perda

            let rejunte_total_arredondado = rejunte_total.ceil();

            self.resultado.push_str(&format!(
                "Cálculo de Rejunte:\nÁrea Total: {:.2} m²\nRevestimento: {:.0}mm x {:.0}mm x {:.0}mm\n\
                Espaçamento das Juntas: {:.2} mm\nCoeficiente de Rejuntamento: {:.2}\nQuantidade de Rejunte Necessária: {:.0} kg\n\n",
                area_total,
                tile_width,
                tile_length,
                tile_thickness,
                joint_spacing,
                rejunte_coeficiente,
                rejunte_total_arredondado,
            ));
        }
    }
}

fn parse_decimal(s: &str) -> Result<Decimal, rust_decimal::Error> {
    let normalized = s.replace(',', ".");
    Decimal::from_str(&normalized)
}

impl CalculadoraLaje {
    fn calcular(&mut self) {
        let mut resultado = String::new();
        let mut total_area = Decimal::ZERO;
        let mut total_elementos = 0u32; // Lajotas ou placas de isopor

        // HashMap para armazenar a contagem de vigas por comprimento
        let mut beams_per_length: HashMap<Decimal, u32> = HashMap::new();

        for (i, ambiente) in self.ambientes.iter().enumerate() {
            let largura_original = match parse_decimal(&ambiente.largura) {
                Ok(v) => v,
                Err(_) => {
                    self.resultado = format!("Largura inválida no ambiente {}", i + 1);
                    return;
                }
            };

            let comprimento_original = match parse_decimal(&ambiente.comprimento) {
                Ok(v) => v,
                Err(_) => {
                    self.resultado = format!("Comprimento inválido no ambiente {}", i + 1);
                    return;
                }
            };

            let (lado_instalacao, lado_perpendicular) = match ambiente.direcao {
                DirecaoLaje::MaiorLado => {
                    if largura_original >= comprimento_original {
                        (largura_original, comprimento_original)
                    } else {
                        (comprimento_original, largura_original)
                    }
                }
                DirecaoLaje::MenorLado => {
                    if largura_original <= comprimento_original {
                        (largura_original, comprimento_original)
                    } else {
                        (comprimento_original, largura_original)
                    }
                }
            };

            let largura_step = match self.tipo_laje {
                TipoLaje::Trelica => LARGURA_TRELICA,
                TipoLaje::Isopor => LARGURA_ISOPOR,
            };

            let comprimento_ajustado = Self::ajustar_dimensao(&lado_instalacao, &COMPRIMENTO_STEP);
            let (largura_ajustada, num_vigas) = Self::reajuste(&lado_perpendicular, &largura_step);

            // Acumular vigas por comprimento
            *beams_per_length.entry(comprimento_ajustado).or_insert(0) += num_vigas;

            let elementos = match self.tipo_laje {
                TipoLaje::Trelica => Self::calc_trelica(&largura_ajustada, &comprimento_ajustado),
                TipoLaje::Isopor => Self::calc_isopor(&num_vigas, &comprimento_ajustado),
            };

            let area_ambiente = largura_original * comprimento_original;
            total_area += area_ambiente;
            total_elementos += elementos;

            resultado.push_str(&format!(
                "Ambiente {}:\nTipo de Laje: {}\nDireção de instalação: {}\nLargura Ajustada: {:.2} m\nComprimento Ajustado: {:.2} m\n\
Vigas: {} de {:.2} m\n{}: {}\nÁrea do Ambiente: {:.2} m²\n\n",
                i + 1,
                if self.tipo_laje == TipoLaje::Trelica {
                    "Treliça"
                } else {
                    "Isopor"
                },
                if ambiente.direcao == DirecaoLaje::MenorLado {
                    "Menor lado"
                } else {
                    "Maior lado"
                },
                largura_ajustada,
                comprimento_ajustado,
                num_vigas,
                comprimento_ajustado,
                if self.tipo_laje == TipoLaje::Trelica {
                    "Lajotas"
                } else {
                    "Placas de isopor"
                },
                elementos,
                area_ambiente
            ));
        }

        resultado.push_str(&format!(
            "Área Total: {:.2} m²\nTotal de Vigas:\n",
            total_area,
        ));

        // Exibir o total de vigas por comprimento
        for (comprimento, quantidade) in beams_per_length.iter() {
            resultado.push_str(&format!(
                "  {} vigas de {:.2} m\n",
                quantidade, comprimento
            ));
        }

        resultado.push_str(&format!(
            "Total de {}: {}\n",
            if self.tipo_laje == TipoLaje::Trelica {
                "Lajotas"
            } else {
                "Placas de isopor"
            },
            total_elementos
        ));

        self.resultado = resultado;
    }

    fn ajustar_dimensao(dimensao: &Decimal, step: &Decimal) -> Decimal {
        (dimensao / step).ceil() * step
    }

    fn reajuste(num: &Decimal, div: &Decimal) -> (Decimal, u32) {
        let count = (num / div).ceil();
        (count * div, count.to_u32().unwrap_or(0))
    }

    fn calc_trelica(largura: &Decimal, comprimento: &Decimal) -> u32 {
        (largura * comprimento * Decimal::from(13))
            .ceil()
            .to_u32()
            .unwrap_or(0)
    }

    fn calc_isopor(vigas: &u32, comprimento: &Decimal) -> u32 {
        (Decimal::from(*vigas) * comprimento / dec!(0.5))
            .ceil()
            .to_u32()
            .unwrap_or(0)
    }
}

impl CalculadoraMateriais {
    fn calcular(&mut self) {
        let (volume, cimento, areia, pedra) = match self.modo_entrada {
            ModoEntradaMateriais::VolumeConcreto => self.calcular_por_volume(),
            ModoEntradaMateriais::QuantidadeMateriais => self.calcular_por_quantidade(),
        };

        if let (Some(volume), Some(cimento), Some(areia), Some(pedra)) =
            (volume, cimento, areia, pedra)
        {
            self.gerar_resultado(volume, cimento, areia, pedra);
        } else {
            self.resultado = "Dados de entrada inválidos".to_string();
        }
    }

    fn calcular_por_volume(
        &self,
    ) -> (
        Option<Decimal>,
        Option<Decimal>,
        Option<Decimal>,
        Option<Decimal>,
    ) {
        let volume = match parse_decimal(&self.volume_concreto) {
            Ok(v) => v,
            Err(_) => {
                return (None, None, None, None);
            }
        };

        let proporcao_total = self.proporcao_total();
        let cimento = volume
            * Decimal::from_str(&self.proporcao_cimento).unwrap_or(dec!(1))
            / proporcao_total;
        let areia = volume
            * Decimal::from_str(&self.proporcao_areia).unwrap_or(dec!(2))
            / proporcao_total;
        let pedra = volume
            * Decimal::from_str(&self.proporcao_pedra).unwrap_or(dec!(2))
            / proporcao_total;

        (Some(volume), Some(cimento), Some(areia), Some(pedra))
    }

    fn calcular_por_quantidade(
        &self,
    ) -> (
        Option<Decimal>,
        Option<Decimal>,
        Option<Decimal>,
        Option<Decimal>,
    ) {
        let cimento = match parse_decimal(&self.quantidade_cimento) {
            Ok(v) => v,
            Err(_) => return (None, None, None, None),
        };

        let areia = if !self.quantidade_areia_m3.is_empty() {
            match parse_decimal(&self.quantidade_areia_m3) {
                Ok(v) => v,
                Err(_) => return (None, None, None, None),
            }
        } else if !self.quantidade_areia_sacos.is_empty() {
            match parse_decimal(&self.quantidade_areia_sacos) {
                Ok(v) => v * PESO_SACO_AREIA / PESO_AREIA_POR_M3,
                Err(_) => return (None, None, None, None),
            }
        } else {
            return (None, None, None, None);
        };

        let pedra = if !self.quantidade_pedra_m3.is_empty() {
            match parse_decimal(&self.quantidade_pedra_m3) {
                Ok(v) => v,
                Err(_) => return (None, None, None, None),
            }
        } else if !self.quantidade_pedra_sacos.is_empty() {
            match parse_decimal(&self.quantidade_pedra_sacos) {
                Ok(v) => v * PESO_SACO_AREIA / PESO_AREIA_POR_M3,
                Err(_) => return (None, None, None, None),
            }
        } else {
            return (None, None, None, None);
        };

        let volume = cimento + areia + pedra;
        (Some(volume), Some(cimento), Some(areia), Some(pedra))
    }

    fn proporcao_total(&self) -> Decimal {
        Decimal::from_str(&self.proporcao_cimento).unwrap_or(dec!(1))
            + Decimal::from_str(&self.proporcao_areia).unwrap_or(dec!(2))
            + Decimal::from_str(&self.proporcao_pedra).unwrap_or(dec!(2))
    }

    fn gerar_resultado(
        &mut self,
        volume: Decimal,
        cimento: Decimal,
        areia: Decimal,
        pedra: Decimal,
    ) {
        let cimento_sacos = (cimento * PESO_AREIA_POR_M3 / PESO_SACO_CIMENTO).ceil();
        let areia_m3 = (areia * dec!(2)).ceil() / dec!(2);
        let areia_sacos = (areia * PESO_AREIA_POR_M3 / PESO_SACO_AREIA).ceil();
        let pedra_m3 = (pedra * dec!(2)).ceil() / dec!(2);
        let pedra_sacos = (pedra * PESO_AREIA_POR_M3 / PESO_SACO_AREIA).ceil();

        self.resultado = format!(
            "Volume de concreto: {:.2} m³\n\
            Cimento: {} sacos de {}kg\n\
            Areia: {:.1} m³ ou {} sacos de {}kg\n\
            Pedra: {:.1} m³ ou {} sacos de {}kg\n\
            Proporção calculada (Cimento:Areia:Pedra): {:.2}:{:.2}:{:.2}",
            volume,
            cimento_sacos,
            PESO_SACO_CIMENTO,
            areia_m3,
            areia_sacos,
            PESO_SACO_AREIA,
            pedra_m3,
            pedra_sacos,
            PESO_SACO_AREIA,
            cimento / cimento,
            areia / cimento,
            pedra / cimento
        );
    }
}

impl CalculadoraBasica {
    fn new() -> Self {
        Self {
            display: "0".to_string(),
            memoria: Decimal::ZERO,
            operacao_pendente: None,
            ultimo_numero: Decimal::ZERO,
            limpar_na_proxima_entrada: false,
            historico: VecDeque::with_capacity(10),
        }
    }

    fn processar_entrada(&mut self, entrada: char) {
        match entrada {
            '0'..='9' | '.' => self.adicionar_digito(entrada),
            '+' | '-' | '*' | '/' => self.definir_operacao(entrada),
            '=' | '\n' => self.calcular_resultado(),
            'c' | 'C' => self.limpar(),
            'm' | 'M' => self.usar_memoria(),
            '\u{8}' => self.apagar(), // Backspace
            _ => {}
        }
    }

    fn adicionar_digito(&mut self, digito: char) {
        if self.limpar_na_proxima_entrada {
            self.display = String::new();
            self.limpar_na_proxima_entrada = false;
        }
        if digito == '.' && self.display.contains('.') {
            return;
        }
        if self.display == "0" && digito != '.' {
            self.display.clear();
        }
        self.display.push(digito);
    }

    fn definir_operacao(&mut self, op: char) {
        self.calcular_resultado();
        self.ultimo_numero = self.display.parse().unwrap_or(Decimal::ZERO);
        self.operacao_pendente = Some(op);
        self.limpar_na_proxima_entrada = true;
    }

    fn calcular_resultado(&mut self) {
        if let Some(op) = self.operacao_pendente {
            let atual: Decimal = self.display.parse().unwrap_or(Decimal::ZERO);
            let resultado = match op {
                '+' => self.ultimo_numero + atual,
                '-' => self.ultimo_numero - atual,
                '*' => self.ultimo_numero * atual,
                '/' => {
                    if atual != Decimal::ZERO {
                        self.ultimo_numero / atual
                    } else {
                        Decimal::ZERO
                    }
                }
                _ => atual,
            };
            self.display = format!("{}", resultado);
            self.historico.push_front(format!(
                "{} {} {} = {}",
                self.ultimo_numero, op, atual, resultado
            ));
            if self.historico.len() > 10 {
                self.historico.pop_back();
            }
            self.operacao_pendente = None;
            self.ultimo_numero = resultado;
        }
    }

    fn limpar(&mut self) {
        self.display = "0".to_string();
        self.operacao_pendente = None;
        self.ultimo_numero = Decimal::ZERO;
        self.limpar_na_proxima_entrada = false;
    }

    fn usar_memoria(&mut self) {
        self.display = format!("{}", self.memoria);
    }

    fn apagar(&mut self) {
        if self.display.len() > 1 {
            self.display.pop();
        } else {
            self.display = "0".to_string();
        }
    }
}

impl CalculadoraForro {
    fn calcular(&mut self) {
        let mut resultado = String::new();
        let mut total_area = Decimal::ZERO;
        let mut total_acabamento = Decimal::ZERO;
        let mut total_emenda_metros = Decimal::ZERO;
        let tamanho_emenda_barra = dec!(6); // Comprimento das barras de emenda
        let mut total_pecas = [0u32; 4]; // [3m, 4m, 5m, 6m]
        let tamanhos = [dec!(3), dec!(4), dec!(5), dec!(6)];

        for (i, ambiente) in self.ambientes.iter().enumerate() {
            let largura = match parse_decimal(&ambiente.largura) {
                Ok(v) => v,
                Err(_) => {
                    self.resultado = format!("Largura inválida no ambiente {}", i + 1);
                    return;
                }
            };

            let comprimento = match parse_decimal(&ambiente.comprimento) {
                Ok(v) => v,
                Err(_) => {
                    self.resultado = format!("Comprimento inválido no ambiente {}", i + 1);
                    return;
                }
            };

            let area = largura * comprimento;
            total_area += area;

            let perimeter = (largura + comprimento) * dec!(2);
            total_acabamento += perimeter;

            let (lado_instalacao, lado_perpendicular) = match self.direcao_global {
                DirecaoForro::MaiorLado => {
                    if largura >= comprimento {
                        (largura, comprimento)
                    } else {
                        (comprimento, largura)
                    }
                }
                DirecaoForro::MenorLado => {
                    if largura <= comprimento {
                        (largura, comprimento)
                    } else {
                        (comprimento, largura)
                    }
                }
            };

            // Calcular o número de fileiras (considerando a largura de 20cm)
            let num_fileiras = (lado_perpendicular / dec!(0.2)).ceil().to_u32().unwrap_or(0);

            let mut pecas_ambiente = [0u32; 4];
            let mut total_juntas = 0u32;

            for _ in 0..num_fileiras {
                let mut comprimento_restante = lado_instalacao;
                let mut num_pecas_por_fileira = 0u32;

                while comprimento_restante > Decimal::ZERO {
                    // Encontrar o maior tamanho possível que seja menor ou igual ao comprimento restante
                    let mut melhor_tamanho = None;
                    for &tamanho in tamanhos.iter().rev() {
                        if tamanho <= comprimento_restante {
                            melhor_tamanho = Some(tamanho);
                            break;
                        }
                    }

                    // Se não houver tamanho adequado, usar o menor tamanho disponível
                    let melhor_tamanho = melhor_tamanho.unwrap_or(tamanhos[0]);

                    let indice = tamanhos.iter().position(|&t| t == melhor_tamanho).unwrap();
                    pecas_ambiente[indice] += 1;
                    total_pecas[indice] += 1;
                    comprimento_restante -= melhor_tamanho;
                    num_pecas_por_fileira += 1;
                }

                if num_pecas_por_fileira > 1 {
                    // Cada junção precisa de emenda
                    total_juntas += num_pecas_por_fileira - 1;
                }
            }

            let emenda_metros_ambiente = Decimal::from(total_juntas) * dec!(0.2);

            total_emenda_metros += emenda_metros_ambiente;

            resultado.push_str(&format!(
                "Ambiente {}: {:.2}m x {:.2}m = {:.2}m²\n",
                i + 1,
                largura,
                comprimento,
                area
            ));
            resultado.push_str(&format!("Perímetro: {:.2}m\n", perimeter));
            resultado.push_str(&format!(
                "Direção de instalação: {}\n",
                if self.direcao_global == DirecaoForro::MaiorLado {
                    "Maior lado"
                } else {
                    "Menor lado"
                }
            ));

            for (i, &peca) in pecas_ambiente.iter().enumerate() {
                if peca > 0 {
                    resultado.push_str(&format!("  Peças de {}m: {}\n", tamanhos[i], peca));
                }
            }

            if emenda_metros_ambiente > Decimal::ZERO {
                resultado.push_str(&format!(
                    "  Quantidade de emenda necessária: {:.2} metros\n",
                    emenda_metros_ambiente
                ));
            } else {
                resultado.push_str("  Emenda: Não é necessário\n");
            }

            resultado.push_str(&format!(
                "  Quantidade de acabamento: {:.2} metros\n",
                perimeter
            ));
            resultado.push_str("\n");
        }

        // Calcular quantidade de barras de emenda
        let total_emenda_barras = (total_emenda_metros / tamanho_emenda_barra)
            .ceil()
            .to_u32()
            .unwrap_or(0);

        resultado.push_str(&format!("Área total dos ambientes: {:.2}m²\n", total_area));
        resultado.push_str("\nTotal de peças necessárias:\n");
        for (i, &peca) in total_pecas.iter().enumerate() {
            if peca > 0 {
                resultado.push_str(&format!("  Peças de {}m: {}\n", tamanhos[i], peca));
            }
        }

        if total_emenda_metros > Decimal::ZERO {
            resultado.push_str(&format!(
                "\nTotal de emenda necessária: {:.2} metros\n",
                total_emenda_metros
            ));
            resultado.push_str(&format!(
                "Quantidade de barras de emenda de {:.2}m: {}\n",
                tamanho_emenda_barra,
                total_emenda_barras
            ));
        } else {
            resultado.push_str("\nEmenda: Não é necessário\n");
        }

        resultado.push_str(&format!(
            "\nQuantidade total de acabamento: {:.2} metros\n",
            total_acabamento
        ));

        self.resultado = resultado;
    }
}

fn key_to_char(key: egui::Key) -> Option<char> {
    match key {
        egui::Key::Num0 => Some('0'),
        egui::Key::Num1 => Some('1'),
        egui::Key::Num2 => Some('2'),
        egui::Key::Num3 => Some('3'),
        egui::Key::Num4 => Some('4'),
        egui::Key::Num5 => Some('5'),
        egui::Key::Num6 => Some('6'),
        egui::Key::Num7 => Some('7'),
        egui::Key::Num8 => Some('8'),
        egui::Key::Num9 => Some('9'),
        egui::Key::Enter => Some('='),
        egui::Key::Minus => Some('-'),
        egui::Key::C => Some('C'),
        egui::Key::M => Some('M'),
        egui::Key::Backspace => Some('\u{8}'),
        _ => None,
    }
}

fn check_for_updates() {
    let update_result = self_update::backends::github::Update::configure()
        .repo_owner("CylexTTY")
        .repo_name("Calculadora_Construcao")
        .bin_name("calculadora_construcao")
        .show_download_progress(true)
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()
        .and_then(|u| u.update());

    match update_result {
        Ok(status) => {
            if status.updated() {
                println!("Atualização aplicada: {} -> {}", status.version(), status.version());
            } else {
                println!("Nenhuma atualização disponível.");
            }
        }
        Err(e) => {
            println!("Erro ao verificar atualizações: {:?}", e);
        }
    }
}

fn main() -> eframe::Result<()> {
    // Verifica e aplica atualizações antes de iniciar o programa
    check_for_updates();

    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(egui::Vec2::new(1024.0, 768.0));
    native_options.resizable = true;
    eframe::run_native(
        "Calculadora de Construção",
        native_options,
        Box::new(|_cc| Box::new(CalculadoraConstrucao::default())),
    )
}
