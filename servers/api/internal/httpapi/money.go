package httpapi

import (
	money "github.com/Rhymond/go-money"
	"github.com/shopspring/decimal"
)

// currencyCode is the one currency this API supports today — a personal,
// single-currency (USD) app. Money is used only at the JSON boundary;
// storage stays decimal.Decimal (see internal/models), so this carries no
// schema change against the already-migrated production database.
const currencyCode = "USD"

// decimalToMoney converts stored decimal.Decimal cents-exact into Money's
// integer minor units via Shift/Round/IntPart — never a float64.
func decimalToMoney(amount decimal.Decimal) money.Money {
	fraction := money.GetCurrency(currencyCode).Fraction
	minorUnits := amount.Shift(int32(fraction)).Round(0).IntPart()
	return *money.New(minorUnits, currencyCode)
}

// moneyToDecimal is the inverse conversion, exact for the same reason. It
// reports false for a zero-value Money (no amount given in the request) or
// one in a currency other than currencyCode.
func moneyToDecimal(m money.Money) (decimal.Decimal, bool) {
	currency := m.Currency()
	if currency == nil || currency.Code != currencyCode {
		return decimal.Decimal{}, false
	}
	return decimal.NewFromInt(m.Amount()).Shift(int32(-currency.Fraction)), true
}
