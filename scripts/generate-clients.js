import {renderJavaScriptVisitor, renderRustVisitor} from '@kinobi-so/renderers';

kinobi.accept(renderJavaScriptVisitor('../idl/testnet/', {
    // Your JavaScript options here
}));

kinobi.accept(renderRustVisitor('../idl/testnet/', {
    // Your Rust options here
}));