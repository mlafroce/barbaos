//! # Módulo de interrupciones externas
//! El *platform level interrupt controller* (PLIC) se encarga de administrar
//! interrupciones externas a la cpu.
//! Utilizaremos el PLIC para atender interrupciones del UART, basándonos en la
//! documentación de QEmu y SiFive

const PLIC_INT_BASE: usize = 0x0C00_0000;
const PLIC_PRIORITY: usize = PLIC_INT_BASE;
const PLIC_INT_ENABLE: usize = PLIC_INT_BASE + 0x2000;
const PLIC_THRESHOLD: usize = PLIC_INT_BASE + 0x20_0000;
const PLIC_CLAIM: usize = PLIC_INT_BASE + 0x20_0004;

/// Habilita una interrupción interna según su id
pub fn enable(id: u32) {
    let enables = PLIC_INT_ENABLE as *mut u32;
    let actual_id = 1 << id;
    unsafe {
        // el registro plic_int_enable es un bitset donde el id es el índice
        // Podemos habilitar interrupciones desde 1 a 31 (0 está hardcodeado)
        enables.write_volatile(enables.read_volatile() | actual_id);
    }
}

/// Asigna una prioridad de 0 a 7
/// 0 no supera ningún threshold, por lo que es como si esetuviera deshabilitado
pub fn set_priority(id: u32, priority: u8) {
    let actual_priority = priority as u32 & 7;
    let priority_reg = PLIC_PRIORITY as *mut u32;
    unsafe {
        priority_reg
            .add(id as usize)
            .write_volatile(actual_priority);
    }
}

/// Threshold global
/// Si la prioridad de la interrupción lanzada es igual o menor el threshold
/// configurado, no se atiende.
pub fn set_threshold(tsh: u8) {
    let actual_tsh = tsh & 7;
    let tsh_reg = PLIC_THRESHOLD as *mut u32;
    unsafe {
        tsh_reg.write_volatile(actual_tsh as u32);
    }
}

/// El PLIC ordena las interrupciones por prioridad, y nos devuelve
/// el ID de la siguiente prioridad.
/// Una vez que se atiende la interrupción, el PLIC no atiende más
/// interrupciones de este dispositivo, por lo que debemos indicar cuándo
/// terminamos de atender la interrupción con la función `complete`
pub fn next_interrupt() -> Option<u32> {
    let claim_reg = PLIC_CLAIM as *const u32;
    let claim_no;
    unsafe {
        claim_no = claim_reg.read_volatile();
    }
    if claim_no == 0 {
        // La interrupción 0 está hardcodeada en 0, indicando que no hay
        // más interrupciones
        None
    } else {
        Some(claim_no)
    }
}

/// Indica que se terminó de atender una interrupción.
/// Debe llamarse con el id devuelto por la función `next_interrupt`
pub fn complete(id: u32) {
    let complete_reg = PLIC_CLAIM as *mut u32;
    unsafe {
        // Es el mismo registro que cuando buscamos `next_interrupt`, pero
        // puede diferenciar si estamos escribiendo o leyendo
        complete_reg.write_volatile(id);
    }
}
