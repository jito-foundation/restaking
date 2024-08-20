import {renderJavaScriptVisitor, renderRustVisitor} from '@kinobi-so/renderers';

kinobi.accept(renderJavaScriptVisitor('clients/js/src/generated', {...}));
kinobi.accept(renderRustVisitor('clients/rust/src/generated', {...}));
