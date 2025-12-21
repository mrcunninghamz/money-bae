use std::cell::OnceCell;
use std::rc::Rc;
use crate::configuration_manager::ConfigurationManager;
use crate::db::PgConnector;
use crate::repositories::*;

pub struct DependencyContainer{
    configuration_manager: OnceCell<ConfigurationManager>,
    pg_connector: OnceCell<Rc<PgConnector>>,
    income_repo: OnceCell<Rc<IncomeRepo>>,
    bill_repo: OnceCell<Rc<BillRepo>>,
    ledger_repo: OnceCell<Rc<LedgerRepo>>,
    pto_repo: OnceCell<Rc<PtoRepo>>,
    pto_plan_repo: OnceCell<Rc<PtoPlanRepo>>,
    holiday_hours_repo: OnceCell<Rc<HolidayHoursRepo>>,
}

impl DependencyContainer {
    pub fn new() -> Self {
        DependencyContainer {
            configuration_manager: OnceCell::new(),
            pg_connector: OnceCell::new(),
            income_repo: OnceCell::new(),
            bill_repo: OnceCell::new(),
            ledger_repo: OnceCell::new(),
            pto_repo: OnceCell::new(),
            pto_plan_repo: OnceCell::new(),
            holiday_hours_repo: OnceCell::new(),
        }
    }

    pub fn configuration_manager(&self) -> &ConfigurationManager {
        self.configuration_manager.get_or_init(|| ConfigurationManager::new())
    }
    
    pub fn pg_connector(&self) -> Rc<PgConnector> {
        Rc::clone(self.pg_connector.get_or_init(|| {
            let config_manager = self.configuration_manager();
            let connection_string = config_manager
                .get_database_connection_string()
                .unwrap_or_else(|| {
                    panic!("database_connection_string not found in configuration file.")
                })
                .to_string();
            Rc::new(PgConnector::new(connection_string))
        }))
    }

    pub fn income_repo(&self) -> Rc<IncomeRepo> {
        Rc::clone(self.income_repo.get_or_init(|| {
            Rc::new(IncomeRepo::new(self.pg_connector()))
        }))
    }

    pub fn bill_repo(&self) -> Rc<BillRepo> {
        Rc::clone(self.bill_repo.get_or_init(|| {
            Rc::new(BillRepo::new(self.pg_connector()))
        }))
    }

    pub fn ledger_repo(&self) -> Rc<LedgerRepo> {
        Rc::clone(self.ledger_repo.get_or_init(|| {
            Rc::new(LedgerRepo::new(self.pg_connector()))
        }))
    }

    pub fn pto_repo(&self) -> Rc<PtoRepo> {
        Rc::clone(self.pto_repo.get_or_init(|| {
            Rc::new(PtoRepo::new(self.pg_connector()))
        }))
    }

    pub fn pto_plan_repo(&self) -> Rc<PtoPlanRepo> {
        Rc::clone(self.pto_plan_repo.get_or_init(|| {
            Rc::new(PtoPlanRepo::new(self.pg_connector()))
        }))
    }

    pub fn holiday_hours_repo(&self) -> Rc<HolidayHoursRepo> {
        Rc::clone(self.holiday_hours_repo.get_or_init(|| {
            Rc::new(HolidayHoursRepo::new(self.pg_connector()))
        }))
    }
}
