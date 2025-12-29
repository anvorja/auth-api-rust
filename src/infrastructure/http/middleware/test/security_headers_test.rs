// src/infrastructure/http/middleware/test/security_headers_test.rs

#[test]
fn test_security_headers_creation() {
    use crate::infrastructure::http::middleware::security_headers_middleware;

    // Test simple que verifica que la función compila correctamente
    // Los tests de integración verificarán el comportamiento real

    // Verificar que el middleware está disponible
    let _middleware = security_headers_middleware;

    // En debug mode, verificar que el permissive también existe
    #[cfg(debug_assertions)]
    {
        use crate::infrastructure::http::middleware::permissive_security_headers_middleware;
        let _permissive = permissive_security_headers_middleware;
    }
}