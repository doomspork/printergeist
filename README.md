# Printergeist

Experimental: Provide browser access to local printers via websockets.

You don't want to use this, I promise.

## List Available Printers

### Request

```json
{
    "type": "list_available_printers"
}
```

### Response

```json
[
    {
        "name": "Brother HL L3210CW series",
        "system_name": "Brother_HL_L3210CW_series"
    },
    {
        "name": "Printer ThermalPrinter",
        "system_name": "Printer_ThermalPrinter"
    }
]
```

## Print

### Request

```json
{
    "data": "...",
    "system_name": "Brother_HL_L3210CW_series",
    "type": "create_print_job"
}
```
