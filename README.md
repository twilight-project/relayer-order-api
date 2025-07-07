# Twilight Relayer Order API

A high-throughput JSON-RPC service for submitting, updating, and managing orders on the Twilight Relayer.

This API suite handles:
- Market and limit order submissions
- Order cancellation and amendment
- Lend order submission and settlement
- Limit order settlement

Built for scale and operational isolation, this service is optimized to handle large volumes of order activity without impacting the performance or reliability of the query layer. 
Ideal for trading bots, exchanges, and institutions interacting directly with the relayer engine.
