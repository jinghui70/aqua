/**
 * GBase resolver — maps GBase JDBC raw physical types to aqua logical DataType.
 * Similar to MySQL/Informix lineage.
 */

module.exports = {
  type: 'gbase',

  resolve: function (column) {
    var raw = column.rawType.toUpperCase();
    var length = column.length;
    var precision = column.precision;
    var scale = column.scale;

    // VARCHAR / VARCHAR2 / CHAR → VARCHAR(n)
    if (raw === 'VARCHAR' || raw === 'VARCHAR2' || raw === 'CHAR') {
      return { dataType: 'VARCHAR', length: length > 0 ? length : undefined };
    }

    // TEXT / CLOB → CLOB
    if (raw === 'TEXT' || raw === 'CLOB') {
      return { dataType: 'CLOB' };
    }

    // TINYINT / SMALLINT → TINYINT
    if (raw === 'TINYINT' || raw === 'SMALLINT') {
      return { dataType: 'TINYINT' };
    }

    // INT / INTEGER → INT
    if (raw === 'INT' || raw === 'INTEGER') {
      return { dataType: 'INT' };
    }

    // BIGINT / SERIAL / BIGSERIAL → LONG
    if (raw === 'BIGINT' || raw === 'SERIAL' || raw === 'BIGSERIAL' || raw === 'INT8') {
      return { dataType: 'LONG' };
    }

    // DECIMAL / NUMERIC → DECIMAL(p,s)
    if (raw === 'DECIMAL' || raw === 'NUMERIC') {
      var p = precision != null && precision > 0 ? precision : undefined;
      var s = scale != null && scale > 0 ? scale : undefined;
      return { dataType: 'DECIMAL', precision: p, scale: s };
    }

    // DATE → DATE
    if (raw === 'DATE') {
      return { dataType: 'DATE' };
    }

    // DATETIME → DATETIME
    if (raw === 'DATETIME') {
      return { dataType: 'DATETIME' };
    }

    // BLOB / BYTEA → BLOB
    if (raw === 'BLOB' || raw === 'BYTEA') {
      return { dataType: 'BLOB' };
    }

    return null;
  }
};
