# Cidacake Client

Cidacake Client é uma aplicação em Rust que interage com o programa Solana `Cidacake`, permitindo a gestão de uma loja de bolos na blockchain Solana (Devnet). O programa suporta três ações principais: vender bolos (`sell`), adicionar estoque (`add_stock`), e atualizar o preço dos bolos (`update_price`).

## Funcionalidades

- **Venda de Bolos (`sell`):** Transfere tokens SPL do comprador para o dono, reduzindo o estoque de bolos.
- **Adição de Estoque (`add_stock`):** Aumenta o estoque de bolos na conta da loja.
- **Atualização de Preço (`update_price`):** Atualiza o preço dos bolos (em lamports).

## Pré-requisitos

- **Rust e Cargo:** Certifique-se de ter o Rust instalado. Você pode instalá-lo com: