// Assembly-level trap handler.
// m_trap_vector va a ser nuestra "función" para atender 
// interrupciones en modo máquina
.section .text
.global m_trap_vector
m_trap_vector:
    // Todas nuestras interrupciones van a caer a este vector
    mret
