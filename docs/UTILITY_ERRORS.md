# ðŸŒ Equipchain Multi-Language Error Mapping

This document provides a mapping of on-chain error codes to human-readable descriptions in multiple languages. This ensures accessibility for users in rural areas and non-English speaking regions (Issue #122).

## Error Code Reference

| Code | ID | Description | Yoruba | Hausa | Igbo | Spanish | French |
|------|----|-------------|--------|-------|------|---------|--------|
| 1 | `MeterNotFound` | Meter not registered. | A kÃ² rÃ­ mita yÃ¬Ã­. | Ba a sami mita ba. | Ahá»¥ghá»‹ mita a. | Medidor no encontrado. | Compteur non trouvÃ©. |
| 5 | `InvalidTokenAmount` | Invalid token amount. | Iye owÃ³ kÃ² tá»Ì. | Adadin kuÉ—i ba daidai ba. | Ego ezughá»‹ oke. | Cantidad de tokens invÃ¡lida. | Montant de jetons invalide. |
| 11 | `TimestampTooOld` | Transaction expired. | Ã€kÃ³kÃ² ti ká»jÃ¡. | Lokaci ya Æ™are. | Oge agwá»¥la. | TransacciÃ³n expirada. | Transaction expirÃ©e. |
| 15 | `MeterNotPaired` | Device not paired. | áº¸Ì€rá» kÃ² tÃ­Ã¬ so pá»Ì€. | Ba a haÉ—a na'ura ba. | Ejiká»taghá»‹ mita. | Dispositivo no vinculado. | Appareil non appairÃ©. |
| 16 | `MeterPaused` | Meter is paused. | Mita ti dÃ¡dÃºrÃ³. | An dakatar da mita. | Akwá»¥sá»‹rá»‹ mita a. | Medidor pausado. | Compteur en pause. |
| 19 | `AccountAlreadyClosed` | Account is closed. | Ã€kÃ Ç¹tÃ¬ ti tÃ¬. | An rufe asusu. | Emechiela akaá»¥ntá»¥ a. | Cuenta ya cerrada. | Compte dÃ©jÃ  fermÃ©. |
| 20 | `InsufficientBalance` | Low balance. | OwÃ³ kÃ² tÃ³. | KuÉ—i ba su isa ba. | Ego ezughá»‹. | Saldo insuficiente. | Solde insuffisant. |
| 22 | `InDispute` | Service in dispute. | Ã€rÃ­yÃ njiyÃ n wÃ . | Akwai jayayya. | E nwere esemokwu. | Servicio en disputa. | Service en litige. |
| 44 | `ProviderNotVerified` | Provider not verified. | OlÃ¹pÃ¨sÃ¨ kÃ² fáº¹sáº¹Ì€ mÃºláº¹Ì€. | Ba a tabbatar da mai samarwa ba. | Akwadoghá»‹ onye na-enye á»rá»¥. | Proveedor no verificado. | Fournisseur non vÃ©rifiÃ©. |
| 49 | `InsufficientXlmReserve` | Gas reserve low. | OwÃ³ gas kÃ² tÃ³. | Gas ya yi Æ™asa. | Ego gas dá»‹ ala. | Reserva de gas insuficiente. | RÃ©serve de gas insuffisante. |

## Backend Integration

The backend service should intercept contract reverts, extract the `u32` error code, and look up the corresponding translation based on the user's localized settings.

### Example Mapping (JSON)
```json
{
  "20": {
    "en": "Insufficient balance to continue service.",
    "yo": "OwÃ³ kÃ² tÃ³ lÃ¡ti táº¹Ì€sÃ­wÃ¡jÃº.",
    "ha": "KuÉ—i ba su isa su ci gaba da sabis ba.",
    "ig": "Ego ezughá»‹ iji gaa n'ihu.",
    "es": "Saldo insuficiente para continuar el servicio.",
    "fr": "Solde insuffisant pour continuer le service."
  }
}
```

**Last Updated**: March 26, 2026