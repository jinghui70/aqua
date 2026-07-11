/**
 * Kingbase (人大金仓) resolver — maps Kingbase JDBC raw physical types to aqua logical DataType.
 * PG-compatible.
 */

module.exports = {
  type: 'kingbase',

  resolve: function (column) {
    var raw = column.rawType.toUpperCase();
    var length = column.length;
    var precision = column.precision;
    var scale = column.scale;

    // VARCHAR / VARCHAR2 / CHAR → VARCHAR(n)
    if (raw === 'VARCHAR' || raw === 'VARCHAR2' || raw === 'CHAR' || raw === 'CHARACTER') {
      return { dataType: 'VARCHAR', length: length > 0 ? length : undefined };
    }

    // TEXT → CLOB
    if (raw === 'TEXT') {
      return { dataType: 'CLOB' };
    }

    // SMALLINT → TINYINT
    if (raw === 'SMALLINT') {
      return { dataType: 'TINYINT' };
    }

    // INTEGER / INT → INT
    if (raw === 'INTEGER' || raw === 'INT' || raw === 'INT4') {
      return { dataType: 'INT' };
    }

    // BIGINT → LONG
    if (raw === 'BIGINT' || raw === 'INT8') {
      return { dataType: 'LONG' };
    }

    // NUMERIC → DECIMAL(p,s)
    if (raw === 'NUMERIC') {
      var p = precision != null && precision > 0 ? precision : undefined;
      var s = scale != null && scale > 0 ? scale : undefined;
      return { dataType: 'DECIMAL', precision: p, scale: s };
    }

    // DATE → DATE
    if (raw === 'DATE') {
      return { dataType: 'DATE' };
    }

    // TIMESTAMP → DATETIME
    if (raw.indexOf('TIMESTAMP') === 0) {
      return { dataType: 'DATETIME' };
    }

    // BYTEA / BLOB → BLOB
    if (raw === 'BYTEA' || raw === 'BLOB') {
      return { dataType: 'BLOB' };
    }

    return null;
  }
};
