use eframe::egui;
use rdr2_xbox_save_converter::{
    convert_archive, convert_pc_to_xbox_offline, create_config_template, default_config_path, load_xbox_slots,
};
use std::path::Path;

const XBOX_KEY_SOURCE: &str =
    "https://community.wemod.com/t/gta-v-save-block-editor-0-0-3-x360-source/2899";
const PC_KEY_SOURCE: &str = "https://github.com/hzhreal/HTOS/blob/31fca60508251e591afe261b927d73c76a7b3503/data/crypto/rstar_crypt.py";

struct ConverterApp {
    input_zip: String,
    output_dir: String,
    config_path: String,
    status: String,
}

impl Default for ConverterApp {
    fn default() -> Self {
        Self {
            input_zip: String::new(),
            output_dir: String::new(),
            config_path: default_config_path().display().to_string(),
            status: "Selecione o ZIP exportado pelo xbcsmgrrev e clique em Analisar save. O programa funciona offline.".into(),
        }
    }
}

impl ConverterApp {
    fn select_input(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Arquivo ZIP", &["zip"])
            .pick_file()
        {
            self.input_zip = path.display().to_string();
            if let Some(parent) = path.parent() {
                self.output_dir = parent.join("converted_pc_saves").display().to_string();
            }
        }
    }

    fn select_output_parent(&mut self) {
        if let Some(parent) = rfd::FileDialog::new().pick_folder() {
            self.output_dir = parent.join("converted_pc_saves").display().to_string();
        }
    }

    fn select_config(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Configuracao TOML", &["toml"])
            .pick_file()
        {
            self.config_path = path.display().to_string();
        }
    }

    fn create_template(&mut self) {
        match create_config_template(Path::new(&self.config_path)) {
            Ok(()) => {
                self.status = format!(
                    "Modelo de configuracao criado em: {}\nPreencha as duas chaves AES consultando as fontes indicadas. Nao publique o arquivo preenchido.",
                    self.config_path
                );
            }
            Err(error) => self.status = format!("Nao foi possivel criar a configuracao: {error}"),
        }
    }

    fn inspect(&mut self) {
        if self.input_zip.trim().is_empty() {
            self.status = "Selecione primeiro o arquivo ZIP do Xbox.".into();
            return;
        }
        match load_xbox_slots(Path::new(&self.input_zip)) {
            Ok(slots) => {
                let mut lines = vec![format!("Analise concluida: {} saves do modo historia encontrados.", slots.len())];
                for slot in slots {
                    lines.push(format!(
                        "{} -> {} | {} | {} bytes",
                        slot.xbox_name,
                        slot.pc_name,
                        slot.title,
                        slot.data.len()
                    ));
                }
                self.status = lines.join("\n");
            }
            Err(error) => self.status = format!("Falha na analise: {error}"),
        }
    }

    fn convert(&mut self) {
        if self.input_zip.trim().is_empty()
            || self.output_dir.trim().is_empty()
            || self.config_path.trim().is_empty()
        {
            self.status = "Informe o ZIP de entrada, a pasta de saida e o caminho do arquivo de chaves.".into();
            return;
        }
        match convert_archive(
            Path::new(&self.input_zip),
            Path::new(&self.output_dir),
            Path::new(&self.config_path),
        ) {
            Ok(manifest) => {
                self.status = format!(
                    "Conversao concluida: {} saves de PC gravados em {}\nValidacoes: RSAV, 10 CHKS, ciclo AES Xbox/PC e igualdade dos dados.",
                    manifest.len(),
                    self.output_dir
                );
            }
            Err(error) => self.status = format!("Conversao recusada: {error}"),
        }
    }
}

impl eframe::App for ConverterApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(context, |ui| {
            ui.heading("Conversor de saves RDR2 - Xbox para PC");
            ui.label("Conversao offline. Nao exige login, nao envia saves e nao inclui chaves de criptografia.");
            ui.label("Atencao: esta versao converte apenas Xbox para PC; nao permite importar saves no Xbox.");
            ui.add_space(10.0);

            egui::Grid::new("paths")
                .num_columns(3)
                .spacing([8.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Xbox ZIP");
                    ui.text_edit_singleline(&mut self.input_zip);
                    if ui.button("Procurar...").clicked() {
                        self.select_input();
                    }
                    ui.end_row();

                    ui.label("Pasta de saida");
                    ui.text_edit_singleline(&mut self.output_dir);
                    if ui.button("Selecionar pasta...").clicked() {
                        self.select_output_parent();
                    }
                    ui.end_row();

                    ui.label("Arquivo de chaves");
                    ui.text_edit_singleline(&mut self.config_path);
                    if ui.button("Procurar...").clicked() {
                        self.select_config();
                    }
                    ui.end_row();
                });

            ui.horizontal(|ui| {
                if ui.button("Criar modelo de chaves").clicked() {
                    self.create_template();
                }
                if ui.button("Analisar save").clicked() {
                    self.inspect();
                }
                if ui.button("Converter e validar").clicked() {
                    self.convert();
                }
            });

            ui.separator();
            ui.label("Fontes publicas das chaves (localize os campos indicados; nenhuma chave esta embutida):");
            ui.horizontal_wrapped(|ui| {
                ui.label("Xbox: campo GTAV= no post de 2013");
                ui.hyperlink_to("Abrir fonte", XBOX_KEY_SOURCE);
                ui.label("  PC: campo PC_KEY no codigo-fonte");
                ui.hyperlink_to("Abrir fonte", PC_KEY_SOURCE);
            });

            ui.separator();
            ui.label("Status");
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(300.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.status)
                            .desired_width(f32::INFINITY)
                            .interactive(false),
                    );
                });
            ui.add_space(6.0);
            ui.small("Seguranca: a pasta de saida nao pode existir. O ZIP original nao e modificado. Falhas de estrutura ou validacao interrompem a conversao.");
        });
    }
}

fn run_cli(arguments: &[String]) -> Option<i32> {
    if arguments.first().map(String::as_str) == Some("--pc-to-xbox-offline") {
        if arguments.len() != 5 {
            eprintln!("usage: rdr2-xbox-save-converter --pc-to-xbox-offline PC_SAVE XBOX_HEADER_TEMPLATE OUTPUT_DIR KEYS.toml");
            return Some(2);
        }
        return Some(match convert_pc_to_xbox_offline(
            Path::new(&arguments[1]),
            Path::new(&arguments[2]),
            Path::new(&arguments[3]),
            Path::new(&arguments[4]),
        ) {
            Ok(manifest) => {
                println!(
                    "OFFLINE prototype generated for {} ({} bytes). No Xbox cloud upload is provided.",
                    manifest.slot, manifest.xbox_data_size
                );
                0
            }
            Err(error) => {
                eprintln!("offline PC-to-Xbox conversion refused: {error}");
                2
            }
        });
    }
    if arguments.first().map(String::as_str) != Some("--convert") {
        return None;
    }
    if arguments.len() != 4 {
        eprintln!("usage: rdr2-xbox-save-converter --convert INPUT.zip OUTPUT_DIR KEYS.toml");
        return Some(2);
    }
    match convert_archive(
        Path::new(&arguments[1]),
        Path::new(&arguments[2]),
        Path::new(&arguments[3]),
    ) {
        Ok(manifest) => {
            println!("converted and verified {} save files", manifest.len());
            Some(0)
        }
        Err(error) => {
            eprintln!("conversion refused: {error}");
            Some(2)
        }
    }
}

fn main() -> eframe::Result {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if let Some(code) = run_cli(&arguments) {
        std::process::exit(code);
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([920.0, 640.0])
            .with_min_inner_size([720.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Conversor de Saves RDR2 - Xbox para PC",
        options,
        Box::new(|_creation_context| Ok(Box::<ConverterApp>::default())),
    )
}
