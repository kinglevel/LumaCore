---
title: Transport Types
category: docs
---
# Transport Types — Address Space Abstraction

**Date:** 2026-03-11
**Status:** Design note (future phase)

## Overview

Generalize the address space beyond DMX universes so LumaCore can natively support multiple transport protocols.

## Hierarchy

```
Patch
└─ AddressSpace
   └─ Fixture
```

## Transport Containers

| Container | Transport | Fixture Addressing |
|-----------|-----------|-------------------|
| Universe  | DMX       | Channel-based     |
| Stream    | Clustr    | Stream-based      |
| Socket    | TCP/UDP/OSC | Socket-based    |
| Endpoint  | HTTP      | HTTP endpoint     |
| Port      | MIDI      | MIDI port         |
| Serial    | ModbusRTU | Serial bus        |

## TransportType → Container Mapping

```
DMX         -> Universe
Clustr      -> Stream
HTTP        -> Endpoint
UDP         -> Socket
TCP         -> Socket
MIDI        -> Port
OSC         -> Socket
ModbusRTU   -> Serial
ModbusTCP   -> Socket
```

## Notes

- Each fixture belongs to a Patch via an AddressSpace
- The container type is determined by the fixture's transport protocol
- Multiple transport types can share a container type (e.g., UDP/TCP/OSC all use Socket)
- This decouples fixture identity from DMX-specific addressing
