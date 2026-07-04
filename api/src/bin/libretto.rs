use anyhow::Result;
use clap::Parser;

// glibc malloc bloats RSS badly under the bulk loader's many-threaded image churn; mimalloc
// keeps it bounded by returning freed pages to the OS promptly.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use libretto::api::{Api, ApiArgs};
use libretto::content_store::ContentStore;
use libretto::ferrari_loader::{FerrariLoader, LoadFerrariArgs};
use libretto::audi_loader::{self, LoadAudiArgs};
use libretto::pdf_text::{self, ExtractPdfTextArgs};
use libretto::settings::Settings;
use libretto::vehicle_importer::{VehicleImporter, VehicleImporterArgs};

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct MigrateArgs {}

#[derive(Parser)]
#[command(version, about, long_about = None)]
enum Cli {
    Api(ApiArgs),
    VehicleImporter(VehicleImporterArgs),
    LoadFerrari(LoadFerrariArgs),
    LoadAudi(LoadAudiArgs),
    ExtractPdfText(ExtractPdfTextArgs),
    Migrate(MigrateArgs),
}


#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let settings = Settings::new()?;

    // Run migrations on every startup (sqlx tracks which have run)
    {
        let content_store = ContentStore::new(&settings);
        content_store.run_migrations(&settings)?;
    }

    match cli {
        Cli::Migrate(_) => {
            println!("Migrations complete.");
            Ok(())
        }
        Cli::VehicleImporter(args) => {
            tokio::task::block_in_place(|| {
                let start = std::time::Instant::now();
                let content_store = ContentStore::new(&settings);
                let vi = VehicleImporter::new(&settings);
                let mut imported = 0;
                for vehicle in &settings.vehicle {
                    content_store.store_vehicle(vehicle)?;
                    if !vehicle.pcss_import {
                        continue;
                    }
                    if let (Some(model), Some(year)) = (&args.model, args.year) {
                        if &vehicle.vehicle != model || year != vehicle.year {
                            continue;
                        }
                    }
                    vi.import(&vehicle.vehicle, vehicle.year, !args.no_text)?;
                    imported += 1;
                }
                println!("Imported {} vehicle(s) in {:.1}s", imported, start.elapsed().as_secs_f64());
                Ok(())
            })
        }
        Cli::Api(args) => {
            let api = Api::new(&settings, &args);
            libretto::api::serve(&settings.api.bind_address, api).await?;
            Ok(())
        }
        Cli::LoadFerrari(args) => {
            let content_store = ContentStore::new(&settings);
            let loader = FerrariLoader::new(content_store);
            loader.load(&args, &settings)
        }
        Cli::LoadAudi(args) => {
            tokio::task::block_in_place(|| audi_loader::run(&settings, &args))
        }
        Cli::ExtractPdfText(args) => {
            tokio::task::block_in_place(|| pdf_text::run(&settings, &args))
        }
    }
}
